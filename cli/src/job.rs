use std::error::Error as StdError;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use async_recursion::async_recursion;
use futures::future::join_all;
use itertools::Itertools;
use log::error;
use log::trace;
use tracing::{error_span, Instrument};

use chord_core::flow::{Flow, ID_PATTERN};
use chord_core::future::fs::read_dir;
use chord_core::future::path::is_dir;
use chord_core::future::task::spawn;
use chord_core::input::JobLoader;
use chord_core::output::{DateTime, JobReporter, Utc};
use chord_core::task::{TaskAsset, TaskId, TaskState};
use chord_core::value::Value;
use chord_flow::{App, TaskIdStruct};
use Error::*;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("no task found")]
    NoTaskFound,

    #[error("job dir error: {0}\n{1}")]
    JobDir(String, std::io::Error),

    #[error("job file error: {0}\n{1}")]
    JobFile(String, chord_input::layout::Error),

    #[error("task file error: {0}\n{1}")]
    TaskFile(String, chord_input::flow::Error),

    #[error("task flow error: {0}\n{1}")]
    TaskFlow(String, chord_core::flow::Error),

    #[error("task case error: {0}\n{1}")]
    TaskCase(String, Box<dyn StdError + Sync + Send>),

    #[error("task report error: {0}\n{1}")]
    Report(String, Box<dyn StdError + Sync + Send>),
}

pub async fn run<P: AsRef<Path>>(
    app: Arc<dyn App>,
    job_loader: Arc<dyn JobLoader>,
    job_reporter: Arc<dyn JobReporter>,
    exec_id: String,
    job_path: P,
    job_path_is_task: bool,
) -> Result<Vec<Box<dyn TaskAsset>>, Error> {
    let task_state_vec = if job_path_is_task {
        task_path_run_to_vec(
            app,
            job_loader,
            job_reporter,
            exec_id,
            job_path.as_ref().to_path_buf(),
            PathBuf::new(),
        )
        .await
    } else {
        job_path_run_recur(
            app,
            job_loader,
            job_reporter,
            exec_id,
            job_path.as_ref().to_path_buf(),
            PathBuf::new(),
        )
        .await
    }?;
    return if task_state_vec.is_empty() {
        Err(NoTaskFound)
    } else {
        Ok(task_state_vec)
    };
}

#[async_recursion]
async fn job_path_run_recur(
    app: Arc<dyn App>,
    job_loader: Arc<dyn JobLoader>,
    job_reporter: Arc<dyn JobReporter>,
    exec_id: String,
    root_path: PathBuf,
    job_sub_path: PathBuf,
) -> Result<Vec<Box<dyn TaskAsset>>, Error> {
    let job_path = root_path.join(job_sub_path.clone());
    let job_path_str = job_path.to_str().unwrap();
    trace!("job path start {}", job_path_str);

    let ctrl_data = if chord_input::layout::exists(&job_path, "chord").await {
        chord_input::layout::load(&job_path, "chord")
            .await
            .map_err(|e| JobFile(job_path.to_str().unwrap().to_string(), e))?
    } else {
        Value::Null
    };

    let serial = ctrl_data["job"]["serial"].as_bool().unwrap_or(false);

    let mut job_dir = read_dir(job_path.clone())
        .await
        .map_err(|e| JobDir(job_path.to_str().unwrap().to_string(), e))?;
    let mut sub_name_vec = Vec::new();
    loop {
        let sub_dir = job_dir
            .next_entry()
            .await
            .map_err(|e| JobDir(job_path.to_str().unwrap().to_string(), e))?;
        if sub_dir.is_none() {
            break;
        }
        let sub_dir = sub_dir.unwrap();

        if !is_dir(sub_dir.path()).await {
            continue;
        }

        let sub_name: String = sub_dir.file_name().to_str().unwrap().into();
        if !ID_PATTERN.is_match(sub_name.as_str()) {
            continue;
        }
        sub_name_vec.push(sub_name);
    }
    sub_name_vec.sort();

    let mut task_asset_vec: Vec<Box<dyn TaskAsset>> = Vec::new();

    for sub_name in sub_name_vec {
        let child_sub_path = job_sub_path.join(sub_name.as_str());
        if serial {
            if sub_dir_is_task_path(root_path.clone(), child_sub_path.clone()).await {
                let asses = task_path_run(
                    app.clone(),
                    job_loader.clone(),
                    job_reporter.clone(),
                    exec_id.clone(),
                    root_path.clone(),
                    child_sub_path,
                )
                .await;
                let not_ok = !asses.state().is_ok();
                task_asset_vec.push(asses);
                if not_ok {
                    break;
                }
            } else {
                let state = job_path_run_recur(
                    app.clone(),
                    job_loader.clone(),
                    job_reporter.clone(),
                    exec_id.clone(),
                    root_path.clone(),
                    child_sub_path,
                )
                .await?;
                let not_ok = state.iter().any(|t| !t.state().is_ok());
                task_asset_vec.extend(state);
                if not_ok {
                    break;
                }
            }
        } else {
            let mut futures = Vec::new();
            if sub_dir_is_task_path(root_path.clone(), child_sub_path.clone()).await {
                let jh = spawn(task_path_run_to_vec(
                    app.clone(),
                    job_loader.clone(),
                    job_reporter.clone(),
                    exec_id.clone(),
                    root_path.clone(),
                    child_sub_path.clone(),
                ));
                futures.push(jh);
            } else {
                let jh = spawn(job_path_run_recur(
                    app.clone(),
                    job_loader.clone(),
                    job_reporter.clone(),
                    exec_id.clone(),
                    root_path.clone(),
                    child_sub_path.clone(),
                ));
                futures.push(jh);
            }
            for state in join_all(futures).await {
                let state = state.expect(
                    format!("spawn Err {}, ", child_sub_path.to_str().unwrap_or("")).as_str(),
                );

                task_asset_vec.extend(state?);
            }
        }
    }

    trace!("job path end {}", job_path_str);
    return Ok(task_asset_vec);
}

async fn task_path_run_to_vec(
    app: Arc<dyn App>,
    job_loader: Arc<dyn JobLoader>,
    job_reporter: Arc<dyn JobReporter>,
    exec_id: String,
    root_path: PathBuf,
    task_sub_path: PathBuf,
) -> Result<Vec<Box<dyn TaskAsset>>, Error> {
    Ok(vec![
        task_path_run(
            app,
            job_loader,
            job_reporter,
            exec_id,
            root_path,
            task_sub_path,
        )
        .await,
    ])
}

pub async fn dir_is_task_path(path: PathBuf) -> bool {
    sub_dir_is_task_path(path, PathBuf::new()).await
}

async fn sub_dir_is_task_path(root_path: PathBuf, sub_path: PathBuf) -> bool {
    let task_path = root_path.join(sub_path);
    chord_input::flow::exists(task_path, "task").await
}

async fn task_path_run(
    app: Arc<dyn App>,
    job_loader: Arc<dyn JobLoader>,
    job_reporter: Arc<dyn JobReporter>,
    exec_id: String,
    root_path: PathBuf,
    task_sub_path: PathBuf,
) -> Box<dyn TaskAsset> {
    let mut task_id = task_sub_path.iter().map(|p| p.to_str().unwrap()).join(".");
    if task_id.is_empty() {
        task_id = root_path
            .iter()
            .last()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();
    }
    let id = Arc::new(TaskIdStruct::new(exec_id, task_id.to_owned()));
    let task_path = root_path.join(task_sub_path);
    chord_flow::CTX_ID
        .scope(
            id.to_string(),
            task_path_run_scope(app, job_loader, job_reporter, task_path, id),
        )
        .await
}

async fn task_path_run_scope(
    app: Arc<dyn App>,
    job_loader: Arc<dyn JobLoader>,
    job_reporter: Arc<dyn JobReporter>,
    task_path: PathBuf,
    id: Arc<TaskIdStruct>,
) -> Box<dyn TaskAsset> {
    trace!("task path start {}", task_path.to_str().unwrap());
    let start = Utc::now();
    let task_asset =
        task_path_run_do(task_path.clone(), id.clone(), app, job_loader, job_reporter).await;
    return match task_asset {
        Err(e) => {
            error!("task path Err {}, {}", task_path.to_str().unwrap(), e);
            let end = Utc::now();
            Box::new(JobTaskAsset {
                id,
                start,
                end,
                state: TaskState::Err(Box::new(e)),
            })
        }
        Ok(asset) => {
            trace!("task path end {}", task_path.to_str().unwrap());
            asset
        }
    };
}

async fn task_path_run_do<P: AsRef<Path>>(
    task_path: P,
    task_id: Arc<TaskIdStruct>,
    app: Arc<dyn App>,
    job_loader: Arc<dyn JobLoader>,
    job_reporter: Arc<dyn JobReporter>,
) -> Result<Box<dyn TaskAsset>, Error> {
    let task_path = Path::new(task_path.as_ref());
    let flow = chord_input::flow::load(task_path, "task")
        .await
        .map_err(|e| TaskFile(task_path.to_str().unwrap().to_string(), e))?;
    let flow = Flow::new(flow, task_path)
        .map_err(|e| TaskFlow(task_path.to_str().unwrap().to_string(), e))?;

    let flow = Arc::new(flow);

    //loader
    let loader = job_loader
        .task(task_id.clone(), flow.clone())
        .await
        .map_err(|e| TaskCase(task_id.task().to_string(), e))?;

    //reporter
    let reporter = job_reporter
        .task(task_id.clone(), flow.clone())
        .await
        .map_err(|e| Report(task_id.task().to_string(), e))?;

    //runner
    let task_asset = chord_flow::TaskRunner::new(loader, reporter, app, flow, task_id.clone())
        .run()
        .instrument(error_span!("task", id=task_id.to_string()))
        .await;

    Ok(task_asset)
}

struct JobTaskAsset {
    id: Arc<TaskIdStruct>,
    start: DateTime<Utc>,
    end: DateTime<Utc>,
    state: TaskState,
}

impl TaskAsset for JobTaskAsset {
    fn id(&self) -> &dyn TaskId {
        self.id.as_ref()
    }

    fn start(&self) -> DateTime<Utc> {
        self.start
    }

    fn end(&self) -> DateTime<Utc> {
        self.end
    }

    fn state(&self) -> &TaskState {
        &self.state
    }
}
