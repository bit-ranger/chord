use std::path::Path;

use async_std::fs::{read_dir, rename};
use async_std::sync::Arc;
use async_std::task::Builder;
use futures::future::join_all;
use futures::StreamExt;
use log::debug;

use chord_common::error::Error;
use chord_common::flow::Flow;
use chord_common::task::TaskState;
use chord_flow::AppContext;
use chord_port::report::mongodb::{Writer, ClientOptions};

pub async fn run<P: AsRef<Path>>(job_path: P,
                                 job_name: String,
                                 exec_id: String,
                                 app_ctx: Arc<dyn AppContext>) -> Result<Vec<TaskState>, Error>{

    debug!("job start {}, {}", job_path.as_ref().to_str().unwrap(), job_name.as_str());
    let writer = Arc::new(Writer::new(ClientOptions::parse("").await?, job_name.as_str(), exec_id.as_str()).await?);
    let mut job_dir = read_dir(job_path.as_ref()).await.unwrap();

    let mut futures = Vec::new();
    loop {
        let task_dir  = job_dir.next().await;
        if task_dir.is_none(){
            break;
        }
        let task_dir = task_dir.unwrap();
        if task_dir.is_err(){
            continue;
        }
        let task_dir = task_dir.unwrap();
        if !task_dir.path().is_dir().await{
            continue;
        }

        let builder = Builder::new()
            .name(task_dir.file_name().to_str().unwrap().into());

        let task_path = job_path.as_ref().join(task_dir.path());
        let jh = builder.spawn(run_task(
            task_path,
            exec_id.clone(),
            app_ctx.clone(),
            writer.clone()))
            .unwrap();
        futures.push(jh);
    }

    let task_state_vec = join_all(futures).await;
    writer.close().await;
    debug!("job end {}, {}", job_path.as_ref().to_str().unwrap(), job_name.as_str());
    return Ok(task_state_vec);
}

async fn run_task<P: AsRef<Path>>(
    task_path: P,
    execution_id: String,
    app_ctx: Arc<dyn AppContext>,
    writer: Arc<Writer>
) -> TaskState
{
    let rt = run_task0(task_path, execution_id.as_str(), app_ctx, writer).await;
    match rt {
        Ok(ts) => ts,
        Err(e) => TaskState::Err(e)
    }
}

async fn run_task0<P: AsRef<Path>>(task_path: P,
                                   _execution_id: &str,
                                   app_ctx: Arc<dyn AppContext>,
                                   writer: Arc<Writer>) -> Result<TaskState, Error> {
    let task_path = Path::new(task_path.as_ref());

    debug!("task start {}", task_path.to_str().unwrap());

    let flow_path = task_path.clone().join("flow.yml");

    let flow = chord_port::load::flow::yml::load(&flow_path)?;
    let flow = Flow::new(flow.clone())?;

    //read
    let data_path = task_path.clone().join("data.csv");
    let mut data_reader = chord_port::load::data::csv::from_path(data_path).await?;

    let task_id = task_path.file_name().unwrap().to_str().unwrap();

    let mut total_task_state = TaskState::Ok(vec![]);
    let size_limit = 99999;
    loop{
        let data = chord_port::load::data::csv::load(&mut data_reader, size_limit)?;
        let data_len = data.len();

        let task_assess = chord_flow::run(app_ctx.clone(), flow.clone(), data, task_id).await;

        //write
        writer.write(task_assess.as_ref());

        match task_assess.state() {
            TaskState::Ok(_) => {},
            TaskState::Fail(_) => {
                total_task_state = TaskState::Fail(vec![]);
            }
            TaskState::Err(e) => {
                return Ok(TaskState::Err(e.clone()));
            }
        }

        if data_len < size_limit {
            break;
        }
    }

    debug!("task end {}", task_path.to_str().unwrap());
    return Ok(total_task_state);
}
