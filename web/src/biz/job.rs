use std::path::Path;

use async_std::fs::read_dir;
use async_std::sync::Arc;
use async_std::task::Builder;
use futures::future::join_all;
use futures::StreamExt;
use log::debug;
use log::info;

use crate::rerr;
use chord_common::error::Error;
use chord_common::flow::Flow;
use chord_common::task::TaskState;
use chord_flow::{FlowContext, TaskIdStruct};
use chord_port::report::elasticsearch::Reporter;
use lazy_static::lazy_static;
use regex::Regex;

pub async fn run<P: AsRef<Path>>(
    job_path: P,
    job_name: String,
    exec_id: String,
    app_ctx: Arc<dyn FlowContext>,
    es_url: String,
    case_batch_size: usize,
) -> Result<Vec<TaskState>, Error> {
    debug!(
        "job start {}, {}",
        job_path.as_ref().to_str().unwrap(),
        job_name.as_str()
    );

    let mut job_dir = read_dir(job_path.as_ref()).await.unwrap();
    let es_index = job_name.clone();

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

        let task_input_dir = job_path.as_ref().join(task_dir.path());
        let jh = builder
            .spawn(run_task(
                task_input_dir,
                exec_id.clone(),
                app_ctx.clone(),
                es_url.clone(),
                es_index.clone(),
                case_batch_size,
            ))
            .unwrap();
        futures.push(jh);
    }

    let task_state_vec = join_all(futures).await;
    debug!(
        "job end {}, {}",
        job_path.as_ref().to_str().unwrap(),
        job_name.as_str()
    );
    return Ok(task_state_vec);
}

async fn run_task<P: AsRef<Path>>(
    input_dir: P,
    exec_id: String,
    app_ctx: Arc<dyn FlowContext>,
    es_url: String,
    es_index: String,
    case_batch_size: usize,
) -> TaskState {
    let input_dir = Path::new(input_dir.as_ref());
    let rt = run_task0(
        input_dir,
        exec_id,
        app_ctx,
        es_url,
        es_index,
        case_batch_size,
    )
    .await;
    match rt {
        Ok(ts) => ts,
        Err(e) => {
            info!("task error {}, {}", input_dir.to_str().unwrap(), e);
            TaskState::Err(e)
        }
    }
}

async fn run_task0<P: AsRef<Path>>(
    task_path: P,
    exec_id: String,
    app_ctx: Arc<dyn FlowContext>,
    es_url: String,
    es_index: String,
    case_batch_size: usize,
) -> Result<TaskState, Error> {
    let task_path = Path::new(task_path.as_ref());
    let task_id = task_path.file_name().unwrap().to_str().unwrap();

    let task_id = Arc::new(TaskIdStruct::new(exec_id, task_id.to_owned())?);
    chord_flow::CTX_ID.with(|tid| tid.replace(task_id.to_string()));
    debug!("task start {}", task_path.to_str().unwrap());

    let flow_path = task_path.clone().join("flow.yml");

    let flow = chord_port::load::flow::yml::load(&flow_path)?;
    let flow = Flow::new(flow)?;

    //read
    let data_file_path = task_path.clone().join("data.csv");
    let mut data_loader = Box::new(chord_port::load::data::csv::Loader::new(data_file_path).await?);

    //write
    let mut assess_reporter = Box::new(Reporter::new(es_url, es_index, task_id.clone()).await?);

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

    debug!("task end {}", task_path.to_str().unwrap());
    return Ok(task_assess.state().clone());
}
