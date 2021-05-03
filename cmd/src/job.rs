use std::path::Path;

use async_std::fs::{read_dir};
use async_std::task::Builder;
use futures::StreamExt;

use chord_common::error::Error;
use chord_common::flow::Flow;
use chord_common::task::{TaskState};
use chord_flow::AppContext;
use futures::future::join_all;
use log::debug;
use async_std::sync::Arc;

pub async fn run<P: AsRef<Path>>(job_path: P,
                                 work_path: P,
                                 execution_id: String,
                                 app_ctx: Arc<dyn AppContext>) -> Vec<TaskState> {
    let job_path_str = job_path.as_ref().to_str().unwrap();

    debug!("job start {}", job_path_str);
    let mut job_dir = read_dir(job_path.as_ref()).await.unwrap();

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

        let builder = Builder::new()
            .name(task_dir.file_name().to_str().unwrap().into());

        let task_path = job_path.as_ref().join(task_dir.path());
        let work_path = std::path::PathBuf::from(work_path.as_ref());
        let jh = builder.spawn(run_task(work_path, task_path, execution_id.clone(), app_ctx.clone())).unwrap();
        futures.push(jh);
    }

    let task_state_vec = join_all(futures).await;
    debug!("job end {}", job_path_str);
    return task_state_vec;
}

async fn run_task<P: AsRef<Path>>(
    work_path: P,
    task_path: P,
    execution_id: String,
    app_ctx: Arc<dyn AppContext>) -> TaskState
{
    let rt = run_task0(work_path, task_path, execution_id.as_str(), app_ctx).await;
    match rt {
        Ok(ts) => ts,
        Err(e) => TaskState::Err(e)
    }
}

async fn run_task0<P: AsRef<Path>>(output_dir: P,
                                   input_dir: P,
                                   _execution_id: &str,
                                   app_ctx: Arc<dyn AppContext>) -> Result<TaskState, Error> {
    let input_dir = Path::new(input_dir.as_ref());
    let task_id = input_dir.file_name().unwrap().to_str().unwrap();
    chord_flow::TASK_ID.with(|tid| tid.replace(task_id.to_owned()));

    debug!("task start {}", input_dir.to_str().unwrap());

    let flow_file = input_dir.clone().join("flow.yml");
    let flow = chord_port::load::flow::yml::load(&flow_file)?;
    let flow = Flow::new(flow.clone())?;
    let flow = Arc::new(flow);

    //read
    let data_file = input_dir.clone().join("data.csv");
    let case_batch_size = 99999;
    let mut data_loader = chord_port::load::data::csv::Loader::new(data_file, case_batch_size).await?;

    //write
    let mut assess_reporter = chord_port::report::csv::Reporter::new(output_dir,
                                                                     task_id, &flow).await?;
    //runner
    let mut runner = chord_flow::Runner::new(app_ctx, flow.clone(), String::from(task_id)).await?;

    let mut total_task_state = TaskState::Ok(vec![]);
    loop {
        let data = data_loader.load().await?;
        let data_len = data.len();

        let task_assess = runner.run(data).await;

        let _ = assess_reporter.write(task_assess.as_ref()).await?;

        match task_assess.state(){
            TaskState::Err(e) => {
                total_task_state = TaskState::Err(e.clone());
                break;
            },
            TaskState::Fail(_) => total_task_state = TaskState::Fail(vec![]),
            _ => ()
        }

        if data_len < case_batch_size {
            break;
        }
    }

    data_loader.close().await?;
    assess_reporter.close().await?;

    debug!("task end {}", input_dir.to_str().unwrap());
    return Ok(total_task_state);
}
