use async_std::fs::read_dir;
use async_std::path::{Path, PathBuf};
use async_std::sync::Arc;
use async_std::task::Builder;
use futures::future::join_all;
use futures::StreamExt;
use log::info;
use log::trace;

use async_recursion::async_recursion;
use chord::flow::{Flow, ID_PATTERN};
use chord::task::TaskState;
use chord::Error;
use chord_flow::{FlowApp, TaskIdSimple};
use chord_input::load;
use chord_output::report::{Factory, ReportFactory};
use itertools::Itertools;

pub async fn run<P: AsRef<Path>>(
    app_ctx: Arc<dyn FlowApp>,
    report_factory: Arc<ReportFactory>,
    exec_id: String,
    job_path: P,
) -> Result<Vec<TaskState>, Error> {
    if dir_is_task(job_path.as_ref().to_path_buf(), PathBuf::new()).await {
        task_run_mock(
            app_ctx,
            report_factory,
            exec_id,
            job_path.as_ref().to_path_buf(),
            PathBuf::new(),
        )
        .await
    } else {
        job_run_recur(
            app_ctx,
            report_factory,
            exec_id,
            job_path.as_ref().to_path_buf(),
            PathBuf::new(),
        )
        .await
    }
}

#[async_recursion]
async fn job_run_recur(
    app_ctx: Arc<dyn FlowApp>,
    report_factory: Arc<ReportFactory>,
    exec_id: String,
    root_path: PathBuf,
    job_sub_path: PathBuf,
) -> Result<Vec<TaskState>, Error> {
    let job_path = root_path.join(job_sub_path.clone());
    let job_path_str = job_path.to_str().unwrap();
    trace!("job start {}", job_path_str);

    let ctrl_data = load::conf::load(&job_path, "chord").await?;
    let serial = ctrl_data["job"]["serial"].as_bool().unwrap_or(false);

    let mut job_dir = read_dir(job_path.clone()).await?;
    let mut sub_name_vec = Vec::new();
    loop {
        let task_dir = job_dir.next().await;
        if task_dir.is_none() {
            break;
        }
        let sub_dir = task_dir.unwrap()?;
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

    let mut futures = Vec::new();
    let mut task_state_vec: Vec<TaskState> = Vec::new();

    for sub_name in sub_name_vec {
        let child_sub_path = job_sub_path.join(sub_name.as_str());
        if serial {
            if dir_is_task(root_path.clone(), child_sub_path.clone()).await {
                let state = task_run(
                    app_ctx.clone(),
                    report_factory.clone(),
                    exec_id.clone(),
                    root_path.clone(),
                    child_sub_path,
                )
                .await;
                task_state_vec.push(state);
            } else {
                let state = job_run_recur(
                    app_ctx.clone(),
                    report_factory.clone(),
                    exec_id.clone(),
                    root_path.clone(),
                    child_sub_path,
                )
                .await?;
                task_state_vec.extend(state)
            }
        } else {
            let builder = Builder::new().name(sub_name.clone());
            if dir_is_task(root_path.clone(), child_sub_path.clone()).await {
                let jh = builder
                    .spawn(task_run_mock(
                        app_ctx.clone(),
                        report_factory.clone(),
                        exec_id.clone(),
                        root_path.clone(),
                        child_sub_path.clone(),
                    ))
                    .expect(
                        format!("task spawn fail {}", child_sub_path.to_str().unwrap_or(""))
                            .as_str(),
                    );
                futures.push(jh);
            } else {
                let jh = builder
                    .spawn(job_run_recur(
                        app_ctx.clone(),
                        report_factory.clone(),
                        exec_id.clone(),
                        root_path.clone(),
                        child_sub_path.clone(),
                    ))
                    .expect(
                        format!("job spawn fail {}", child_sub_path.to_str().unwrap_or(""))
                            .as_str(),
                    );
                futures.push(jh);
            }
        }
    }

    if !serial {
        for state in join_all(futures).await {
            task_state_vec.extend(state?);
        }
    }

    trace!("job end {}", job_path_str);
    return Ok(task_state_vec);
}

async fn task_run_mock(
    app_ctx: Arc<dyn FlowApp>,
    report_factory: Arc<ReportFactory>,
    exec_id: String,
    root_path: PathBuf,
    task_sub_path: PathBuf,
) -> Result<Vec<TaskState>, Error> {
    Ok(vec![
        task_run(app_ctx, report_factory, exec_id, root_path, task_sub_path).await,
    ])
}

async fn dir_is_task(root_path: PathBuf, sub_path: PathBuf) -> bool {
    let task_path = root_path.join(sub_path);
    load::flow::exists(task_path, "flow").await
}

async fn task_run(
    app_ctx: Arc<dyn FlowApp>,
    report_factory: Arc<ReportFactory>,
    exec_id: String,
    root_path: PathBuf,
    task_sub_path: PathBuf,
) -> TaskState {
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
    let task_id = Arc::new(TaskIdSimple::new(exec_id, task_id.to_owned()));
    let task_path = root_path.join(task_sub_path);
    chord_flow::CTX_ID.with(|tid| tid.replace(task_id.to_string()));
    trace!("task start {}", task_path.to_str().unwrap());
    let task_state = task_run0(task_path.clone(), task_id, app_ctx, report_factory).await;
    return if let Err(e) = task_state {
        info!("task error {}, {}", task_path.to_str().unwrap(), e);
        TaskState::Err(e.clone())
    } else {
        trace!("task end {}", task_path.to_str().unwrap());
        task_state.unwrap()
    };
}

async fn task_run0<P: AsRef<Path>>(
    task_path: P,
    task_id: Arc<TaskIdSimple>,
    app_ctx: Arc<dyn FlowApp>,
    report_factory: Arc<ReportFactory>,
) -> Result<TaskState, Error> {
    let task_path = Path::new(task_path.as_ref());
    let flow = load::flow::load(task_path, "flow").await?;
    let flow = Flow::new(flow)?;

    //read
    let case_store = Box::new(load::data::Store::new(task_path.clone()).await?);

    //write
    let assess_reporter = report_factory.create(task_id.clone()).await?;

    //runner
    let mut runner = chord_flow::TaskRunner::new(
        case_store,
        assess_reporter,
        app_ctx,
        Arc::new(flow),
        task_id.clone(),
    )
    .await?;

    let task_assess = runner.run().await?;

    return match task_assess.state() {
        TaskState::Ok => Ok(TaskState::Ok),
        TaskState::Fail(c) => Ok(TaskState::Fail(c.clone())),
        TaskState::Err(e) => Ok(TaskState::Err(e.clone())),
    };
}
