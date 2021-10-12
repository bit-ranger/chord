use async_std::fs::read_dir;
use async_std::path::Path;
use async_std::sync::Arc;
use async_std::task::Builder;
use futures::future::join_all;
use futures::StreamExt;
use log::info;
use log::trace;

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
    serial: bool,
) -> Result<Vec<TaskState>, Error> {
    let job_path_str = job_path.as_ref().to_str().unwrap();

    trace!("job start {}", job_path_str);
    let mut job_dir = read_dir(job_path.as_ref()).await?;

    let mut futures = Vec::new();
    let mut task_state_vec = Vec::new();
    loop {
        let task_dir = job_dir.next().await;
        if task_dir.is_none() {
            break;
        }
        let task_dir = task_dir.unwrap()?;
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

        let builder = Builder::new().name(task_name.clone());

        let task_input_dir = job_path.as_ref().join(task_dir.path());
        let trf = task_run(
            task_input_dir,
            exec_id.clone(),
            app_ctx.clone(),
            report_factory.clone(),
        );

        if serial {
            let state = builder.blocking(trf);
            task_state_vec.push(state);
        } else {
            let jh = builder
                .spawn(trf)
                .expect(format!("task spawn fail {}", task_name).as_str());
            futures.push(jh);
        }
    }

    if !serial {
        task_state_vec = join_all(futures).await;
    }

    trace!("job end {}", job_path_str);
    return Ok(task_state_vec);
}

async fn task_run<P: AsRef<Path>>(
    task_path: P,
    exec_id: String,
    app_ctx: Arc<dyn FlowApp>,
    report_factory: Arc<ReportFactory>,
) -> TaskState {
    let task_path = Path::new(task_path.as_ref());
    trace!("task start {}", task_path.to_str().unwrap());
    let task_state = task_run0(task_path, exec_id, app_ctx, report_factory).await;
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
    exec_id: String,
    app_ctx: Arc<dyn FlowApp>,
    report_factory: Arc<ReportFactory>,
) -> Result<TaskState, Error> {
    let task_path = Path::new(task_path.as_ref());
    let task_id = task_path.file_name().unwrap().to_str().unwrap();

    let task_id = Arc::new(TaskIdSimple::new(exec_id, task_id.to_owned())?);
    chord_flow::CTX_ID.with(|tid| tid.replace(task_id.to_string()));
    trace!("task start {}", task_path.to_str().unwrap());

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
