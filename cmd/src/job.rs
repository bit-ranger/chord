use std::path::Path;

use async_std::fs::read_dir;
use async_std::sync::Arc;
use async_std::task::Builder;
use futures::future::join_all;
use futures::StreamExt;
use log::debug;
use log::info;

use chord_common::error::Error;
use chord_common::flow::Flow;
use chord_common::task::TaskState;
use chord_flow::{FlowContext, TaskIdStruct};

pub async fn run<P: AsRef<Path>>(
    input_dir: P,
    output_dir: P,
    exec_id: String,
    app_ctx: Arc<dyn FlowContext>,
) -> Vec<TaskState> {
    let job_path_str = input_dir.as_ref().to_str().unwrap();

    debug!("job start {}", job_path_str);
    let mut job_dir = read_dir(input_dir.as_ref()).await.unwrap();

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

        let builder = Builder::new().name(task_dir.file_name().to_str().unwrap().into());

        let task_input_dir = input_dir.as_ref().join(task_dir.path());
        let output_dir = std::path::PathBuf::from(output_dir.as_ref());
        let jh = builder
            .spawn(run_task(
                task_input_dir,
                output_dir,
                exec_id.clone(),
                app_ctx.clone(),
            ))
            .unwrap();
        futures.push(jh);
    }

    let task_state_vec = join_all(futures).await;
    debug!("job end {}", job_path_str);
    return task_state_vec;
}

async fn run_task<P: AsRef<Path>>(
    input_dir: P,
    output_dir: P,
    exec_id: String,
    app_ctx: Arc<dyn FlowContext>,
) -> TaskState {
    let input_dir = Path::new(input_dir.as_ref());
    let rt = run_task0(input_dir, output_dir, exec_id, app_ctx).await;
    match rt {
        Ok(ts) => ts,
        Err(e) => {
            info!("task error {}, {}", input_dir.to_str().unwrap(), e);
            TaskState::Err(e)
        }
    }
}

async fn run_task0<I: AsRef<Path>, O: AsRef<Path>>(
    input_dir: I,
    output_dir: O,
    exec_id: String,
    app_ctx: Arc<dyn FlowContext>,
) -> Result<TaskState, Error> {
    let input_dir = Path::new(input_dir.as_ref());
    let task_id = input_dir.file_name().unwrap().to_str().unwrap();

    let task_id = Arc::new(TaskIdStruct::new(exec_id, task_id.to_owned())?);
    chord_flow::CTX_ID.with(|tid| tid.replace(task_id.to_string()));

    debug!("task start {}", input_dir.to_str().unwrap());

    let flow_file = input_dir.clone().join("flow.yml");
    let flow = chord_input::load::flow::yml::load(&flow_file)?;
    let flow = Flow::new(flow)?;

    //read
    let data_file_path = input_dir.clone().join("case.csv");
    let data_loader = Box::new(chord_input::load::data::csv::Loader::new(data_file_path).await?);

    //write
    let assess_reporter =
        Box::new(chord_output::report::csv::Reporter::new(output_dir, &flow, task_id.clone()).await?);

    //runner
    let mut runner = chord_flow::Runner::new(
        data_loader,
        assess_reporter,
        app_ctx,
        Arc::new(flow),
        task_id.clone(),
    )
    .await?;

    let task_assess = runner.run().await?;

    debug!("task end {}", input_dir.to_str().unwrap());

    return Ok(task_assess.state().clone());
}
