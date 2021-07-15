use async_std::fs::read_dir;
use async_std::path::Path;
use async_std::sync::Arc;
use async_std::task::Builder;
use futures::future::join_all;
use futures::StreamExt;
use log::info;
use log::trace;

use crate::conf::Config;
use chord::flow::{Flow, ID_PATTERN};
use chord::task::TaskState;
use chord::Error;
use chord_flow::{Context, TaskIdSimple};
use chord_output::report::{Factory, ReportFactory};

pub async fn run<P: AsRef<Path>>(
    input_dir: P,
    task_vec: Option<Vec<String>>,
    exec_id: String,
    app_ctx: Arc<dyn Context>,
    conf: &Config,
) -> Result<Vec<TaskState>, Error> {
    let job_path_str = input_dir.as_ref().to_str().unwrap();

    trace!("job start {}", job_path_str);
    let mut job_dir = read_dir(input_dir.as_ref()).await.unwrap();

    let report_factory = ReportFactory::new(conf.report()).await?;
    let report_factory = Arc::new(report_factory);

    let mut futures = Vec::new();
    loop {
        let task_dir = job_dir.next().await;
        if task_dir.is_none() {
            break;
        }
        let task_dir = task_dir.unwrap();
        if task_dir.is_err() {
            continue;
        }
        let task_dir = task_dir.unwrap();
        if !task_dir.path().is_dir().await {
            continue;
        }

        let task_name: String = task_dir.file_name().to_str().unwrap().into();
        if !ID_PATTERN.is_match(task_name.as_str()) {
            continue;
        }
        if let Some(t) = &task_vec {
            if !t.contains(&task_name) {
                continue;
            }
        }

        let builder = Builder::new().name(task_name);

        let task_input_dir = input_dir.as_ref().join(task_dir.path());
        let jh = builder
            .spawn(run_task(
                task_input_dir,
                exec_id.clone(),
                app_ctx.clone(),
                report_factory.clone(),
            ))
            .unwrap();
        futures.push(jh);
    }

    let task_state_vec = join_all(futures).await;
    trace!("job end {}", job_path_str);
    return Ok(task_state_vec);
}

async fn run_task<P: AsRef<Path>>(
    input_dir: P,
    exec_id: String,
    app_ctx: Arc<dyn Context>,
    report_factory: Arc<ReportFactory>,
) -> TaskState {
    let input_dir = Path::new(input_dir.as_ref());
    let rt = run_task0(input_dir, exec_id, app_ctx, report_factory).await;
    match rt {
        Ok(ts) => {
            trace!("task end {}", input_dir.to_str().unwrap());
            ts
        }
        Err(e) => {
            info!("task error {}, {}", input_dir.to_str().unwrap(), e);
            TaskState::Err(e)
        }
    }
}

async fn run_task0<I: AsRef<Path>>(
    input_dir: I,
    exec_id: String,
    app_ctx: Arc<dyn Context>,
    report_factory: Arc<ReportFactory>,
) -> Result<TaskState, Error> {
    let input_dir = Path::new(input_dir.as_ref());
    let task_id = input_dir.file_name().unwrap().to_str().unwrap();

    let task_id = Arc::new(TaskIdSimple::new(exec_id, task_id.to_owned())?);
    chord_flow::CTX_ID.with(|tid| tid.replace(task_id.to_string()));

    trace!("task start {}", input_dir.to_str().unwrap());

    let flow_file = input_dir.clone().join("flow.yml");
    let flow = chord_input::load::flow::yml::load(&flow_file)?;
    let flow = Flow::new(flow)?;

    //read
    let data_file_path = input_dir.clone().join("case.csv");
    let data_loader = Box::new(chord_input::load::data::csv::Loader::new(data_file_path).await?);

    //write
    let assess_reporter = report_factory.create(task_id.clone()).await?;

    //runner
    let mut runner = chord_flow::TaskRunner::new(
        data_loader,
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
