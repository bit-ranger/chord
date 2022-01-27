use async_recursion::async_recursion;
use async_std::fs::read_dir;
use async_std::path::{Path, PathBuf};
use async_std::sync::Arc;
use async_std::task::Builder;
use futures::future::join_all;
use futures::StreamExt;
use itertools::Itertools;
use log::error;
use log::trace;

use chord_core::flow::{Flow, ID_PATTERN};
use chord_core::output::{DateTime, Factory, Utc};
use chord_core::task::{TaskAssess, TaskId, TaskState};
use chord_core::value::Value;
use chord_flow::{FlowApp, TaskIdSimple};
use chord_input::load;
use Error::*;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("no task found")]
    NoTaskFound,

    #[error("job dir error: {0}\n{1}")]
    JobDir(String, std::io::Error),

    #[error("job file error: {0}\n{1}")]
    JobFile(String, load::conf::Error),

    #[error("task file error: {0}\n{1}")]
    TaskFile(String, load::flow::Error),

    #[error("task flow error: {0}\n{1}")]
    TaskFlow(String, chord_core::flow::Error),

    #[error("task case error: {0}\n{1}")]
    TaskCase(String, Box<dyn std::error::Error + Sync + Send>),

    #[error("task report error: {0}\n{1}")]
    Report(String, Box<dyn std::error::Error + Sync + Send>),
}

pub async fn run<P: AsRef<Path>>(
    app_ctx: Arc<dyn FlowApp>,
    report_factory: Arc<dyn Factory>,
    exec_id: String,
    job_path: P,
) -> Result<Vec<Box<dyn TaskAssess>>, Error> {
    let task_state_vec = if dir_is_task_path(job_path.as_ref().to_path_buf(), PathBuf::new()).await
    {
        task_path_run_cast_vec(
            app_ctx,
            report_factory,
            exec_id,
            job_path.as_ref().to_path_buf(),
            PathBuf::new(),
        )
        .await
    } else {
        job_path_run_recur(
            app_ctx,
            report_factory,
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
    app_ctx: Arc<dyn FlowApp>,
    report_factory: Arc<dyn Factory>,
    exec_id: String,
    root_path: PathBuf,
    job_sub_path: PathBuf,
) -> Result<Vec<Box<dyn TaskAssess>>, Error> {
    let job_path = root_path.join(job_sub_path.clone());
    let job_path_str = job_path.to_str().unwrap();
    trace!("job path start {}", job_path_str);

    let ctrl_data = if load::conf::exists(&job_path, "chord").await {
        load::conf::load(&job_path, "chord")
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
        let sub_dir = job_dir.next().await;
        if sub_dir.is_none() {
            break;
        }
        let sub_dir = sub_dir
            .unwrap()
            .map_err(|e| JobDir(job_path.to_str().unwrap().to_string(), e))?;

        if !sub_dir.path().is_dir().await {
            continue;
        }

        let sub_name: String = sub_dir.file_name().to_str().unwrap().into();
        if !ID_PATTERN.is_match(sub_name.as_str()) {
            continue;
        }
        sub_name_vec.push(sub_name);
    }
    sub_name_vec.sort();

    let mut task_assess_vec: Vec<Box<dyn TaskAssess>> = Vec::new();

    for sub_name in sub_name_vec {
        let child_sub_path = job_sub_path.join(sub_name.as_str());
        if serial {
            if dir_is_task_path(root_path.clone(), child_sub_path.clone()).await {
                let asses = task_path_run(
                    app_ctx.clone(),
                    report_factory.clone(),
                    exec_id.clone(),
                    root_path.clone(),
                    child_sub_path,
                )
                .await;
                let not_ok = !asses.state().is_ok();
                task_assess_vec.push(asses);
                if not_ok {
                    break;
                }
            } else {
                let state = job_path_run_recur(
                    app_ctx.clone(),
                    report_factory.clone(),
                    exec_id.clone(),
                    root_path.clone(),
                    child_sub_path,
                )
                .await?;
                let not_ok = state.iter().any(|t| !t.state().is_ok());
                task_assess_vec.extend(state);
                if not_ok {
                    break;
                }
            }
        } else {
            let mut futures = Vec::new();
            let builder = Builder::new().name(sub_name.clone());
            if dir_is_task_path(root_path.clone(), child_sub_path.clone()).await {
                let jh = builder
                    .spawn(task_path_run_cast_vec(
                        app_ctx.clone(),
                        report_factory.clone(),
                        exec_id.clone(),
                        root_path.clone(),
                        child_sub_path.clone(),
                    ))
                    .expect(
                        format!(
                            "task path Err {}, spawn",
                            child_sub_path.to_str().unwrap_or("")
                        )
                        .as_str(),
                    );
                futures.push(jh);
            } else {
                let jh = builder
                    .spawn(job_path_run_recur(
                        app_ctx.clone(),
                        report_factory.clone(),
                        exec_id.clone(),
                        root_path.clone(),
                        child_sub_path.clone(),
                    ))
                    .expect(
                        format!(
                            "job path Err {}, spawn",
                            child_sub_path.to_str().unwrap_or("")
                        )
                        .as_str(),
                    );
                futures.push(jh);
            }
            for state in join_all(futures).await {
                task_assess_vec.extend(state?);
            }
        }
    }

    trace!("job path end {}", job_path_str);
    return Ok(task_assess_vec);
}

async fn task_path_run_cast_vec(
    app_ctx: Arc<dyn FlowApp>,
    report_factory: Arc<dyn Factory>,
    exec_id: String,
    root_path: PathBuf,
    task_sub_path: PathBuf,
) -> Result<Vec<Box<dyn TaskAssess>>, Error> {
    Ok(vec![
        task_path_run(app_ctx, report_factory, exec_id, root_path, task_sub_path).await,
    ])
}

async fn dir_is_task_path(root_path: PathBuf, sub_path: PathBuf) -> bool {
    let task_path = root_path.join(sub_path);
    load::flow::exists(task_path, "task").await
}

async fn task_path_run(
    app_ctx: Arc<dyn FlowApp>,
    report_factory: Arc<dyn Factory>,
    exec_id: String,
    root_path: PathBuf,
    task_sub_path: PathBuf,
) -> Box<dyn TaskAssess> {
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
    let id = Arc::new(TaskIdSimple::new(exec_id, task_id.to_owned()));
    let task_path = root_path.join(task_sub_path);
    chord_flow::CTX_ID.with(|tid| tid.replace(id.to_string()));
    trace!("task path start {}", task_path.to_str().unwrap());
    let start = Utc::now();
    let task_assess = task_path_run0(task_path.clone(), id.clone(), app_ctx, report_factory).await;
    return match task_assess {
        Err(e) => {
            error!("task path Err {}, {}", task_path.to_str().unwrap(), e);
            let end = Utc::now();
            Box::new(JobTaskAssess {
                id,
                start,
                end,
                state: TaskState::Err(Box::new(e)),
            })
        }
        Ok(assess) => {
            trace!("task path end {}", task_path.to_str().unwrap());
            assess
        }
    };
}

async fn task_path_run0<P: AsRef<Path>>(
    task_path: P,
    task_id: Arc<TaskIdSimple>,
    app_ctx: Arc<dyn FlowApp>,
    report_factory: Arc<dyn Factory>,
) -> Result<Box<dyn TaskAssess>, Error> {
    let task_path = Path::new(task_path.as_ref());
    let flow = load::flow::load(task_path, "task")
        .await
        .map_err(|e| TaskFile(task_path.to_str().unwrap().to_string(), e))?;
    let flow = Flow::new(flow, task_path)
        .map_err(|e| TaskFlow(task_path.to_str().unwrap().to_string(), e))?;

    //read
    let case_store = Box::new(
        load::case::Store::new(task_path.clone())
            .await
            .map_err(|e| TaskCase(task_id.task().to_string(), e))?,
    );

    let flow = Arc::new(flow);

    //write
    let assess_reporter = report_factory
        .create(task_id.clone(), flow.clone())
        .await
        .map_err(|e| Report(task_id.task().to_string(), e))?;

    //runner
    let task_assess =
        chord_flow::TaskRunner::new(case_store, assess_reporter, app_ctx, flow, task_id.clone())
            .run()
            .await;

    Ok(task_assess)
}

struct JobTaskAssess {
    id: Arc<TaskIdSimple>,
    start: DateTime<Utc>,
    end: DateTime<Utc>,
    state: TaskState,
}

impl TaskAssess for JobTaskAssess {
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
