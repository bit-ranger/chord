use async_std::fs::read_dir;
use async_std::path::{Path, PathBuf};
use async_std::sync::Arc;
use async_std::task::Builder;
use futures::future::join_all;
use futures::StreamExt;
use log::info;
use log::trace;

use crate::load_conf;
use async_recursion::async_recursion;
use chord::flow::{Flow, ID_PATTERN};
use chord::task::TaskState;
use chord::Error;
use chord_flow::{FlowApp, TaskIdSimple};
use chord_output::report::{Factory, ReportFactory};

pub async fn run<P: AsRef<Path>>(
    report_factory: Arc<ReportFactory>,
    job_path: P,
    task_vec: Option<Vec<String>>,
    exec_id: String,
    app_ctx: Arc<dyn FlowApp>,
) -> Result<Vec<TaskState>, Error> {
    job_run_recur(
        report_factory,
        job_path.as_ref().to_path_buf(),
        task_vec,
        exec_id,
        app_ctx,
    )
    .await
}

#[async_recursion]
async fn job_run_recur(
    report_factory: Arc<ReportFactory>,
    job_path: PathBuf,
    sub_vec: Option<Vec<String>>,
    exec_id: String,
    app_ctx: Arc<dyn FlowApp>,
) -> Result<Vec<TaskState>, Error> {
    let job_path_str = job_path.to_str().unwrap();
    trace!("job start {}", job_path_str);

    let ctrl_path = job_path.join(".chord.yml");
    let ctrl_data = load_conf(ctrl_path).await?;
    let serial = ctrl_data["job"]["serial"].as_bool().unwrap_or(false);
    let job_name_suffix = ctrl_data["job"]["suffix"].as_str().unwrap_or("_job");

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
        if let Some(t) = &sub_vec {
            if !t.contains(&sub_name) {
                continue;
            }
        }
        sub_name_vec.push(sub_name);
    }
    sub_name_vec.sort();

    let mut futures = Vec::new();
    let mut task_state_vec: Vec<TaskState> = Vec::new();

    for sub_name in sub_name_vec {
        let sub_dir = job_path.join(sub_name.as_str());

        if serial {
            if sub_name.ends_with(job_name_suffix) {
                let state = job_run_recur(
                    report_factory.clone(),
                    job_path.join(sub_name.as_str()),
                    None,
                    exec_id.clone(),
                    app_ctx.clone(),
                )
                .await?;
                task_state_vec.extend(state)
            } else {
                let state = task_run(
                    sub_dir,
                    exec_id.clone(),
                    app_ctx.clone(),
                    report_factory.clone(),
                )
                .await;
                task_state_vec.push(state);
            }
        } else {
            let builder = Builder::new().name(sub_name.clone());
            if sub_name.ends_with(job_name_suffix) {
                let jh = builder
                    .spawn(job_run_recur(
                        report_factory.clone(),
                        job_path.join(sub_name.as_str()),
                        None,
                        exec_id.clone(),
                        app_ctx.clone(),
                    ))
                    .expect(format!("job spawn fail {}", sub_name).as_str());
                futures.push(jh);
            } else {
                let jh = builder
                    .spawn(task_run_mock(
                        sub_dir,
                        exec_id.clone(),
                        app_ctx.clone(),
                        report_factory.clone(),
                    ))
                    .expect(format!("task spawn fail {}", sub_name).as_str());
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

async fn task_run_mock<P: AsRef<Path>>(
    task_path: P,
    exec_id: String,
    app_ctx: Arc<dyn FlowApp>,
    report_factory: Arc<ReportFactory>,
) -> Result<Vec<TaskState>, Error> {
    Ok(vec![
        task_run(task_path, exec_id, app_ctx, report_factory).await,
    ])
}

async fn task_run<P: AsRef<Path>>(
    task_path: P,
    exec_id: String,
    app_ctx: Arc<dyn FlowApp>,
    report_factory: Arc<ReportFactory>,
) -> TaskState {
    let task_path = Path::new(task_path.as_ref());
    let task_id = task_path.file_name().unwrap().to_str().unwrap();
    let task_id = Arc::new(TaskIdSimple::new(exec_id, task_id.to_owned()));
    chord_flow::CTX_ID.with(|tid| tid.replace(task_id.to_string()));
    trace!("task start {}", task_path.to_str().unwrap());
    let task_state = task_run0(task_path, task_id, app_ctx, report_factory).await;
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
    let flow_file = task_path.clone().join("flow.yml");
    let flow = chord_input::load::flow::yml::load(&flow_file)?;
    let flow = Flow::new(flow)?;

    //read
    let case_store = Box::new(chord_input::load::data::csv::Store::new(task_path.clone()).await?);

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
        TaskState::Fail => Ok(TaskState::Fail),
        TaskState::Err(e) => Ok(TaskState::Err(e.clone())),
    };
}
