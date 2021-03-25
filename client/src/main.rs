use std::env;
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::SystemTime;

use async_std::fs;
use async_std::task::Builder;
use futures::future::join_all;
use log::info;

use common::{cause, err};
use common::error::Error;
use common::flow::Flow;
use common::task::TaskState;
use flow::AppContext;
use point::PointRunnerDefault;
use futures::StreamExt;

mod logger;
// mod mdc;


#[async_std::main]
async fn main() -> Result<(),Error> {
    let args: Vec<_> = env::args().collect();
    let mut opts = getopts::Options::new();
    opts.reqopt("j", "job", "job path", "job");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(e) => {
            println!("{}", opts.short_usage("chord"));
            return err!("arg", e.to_string().as_str());
        }
    };

    let job_path = matches.opt_str("j").unwrap();
    let job_path = Path::new(&job_path);
    if !job_path.is_dir(){
        panic!("job path is not a dir {}", job_path.to_str().unwrap());
    }

    let duration = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH).unwrap();
    let execution_id = duration.as_millis().to_string();

    let log_path = job_path.clone().join("log.log");
    let switch = Arc::new(AtomicBool::new(true));
    let log_handler = logger::init(execution_id.clone(),
                                   String::from(".*"),
                              log_path.to_str().map(|s| s.to_owned()).unwrap(),
                                   switch.clone()
    ).await?;



    let task_state_vec = run_job(job_path, execution_id.as_str()).await;

    switch.store(false, Ordering::SeqCst);
    let _ = log_handler.join();

    let et = task_state_vec.iter().filter(|t| !t.is_ok()).last();
    return match et {
        Some(et) => {
            match et {
                TaskState::Ok(_) => Ok(()),
                TaskState::Err(e) => cause!("task", e.to_string().as_str(), e.clone()),
                TaskState::Fail(_) => err!("task", "fail")
            }
        },
        None => Ok(())
    };
}


pub async fn run_job<P: AsRef<Path>>(job_path: P,
                                     execution_id: &str) -> Vec<TaskState>{
    let job_path_str = job_path.as_ref().to_str().unwrap();

    info!("job start {}", job_path_str);
    let mut job_dir = fs::read_dir(job_path.as_ref()).await.unwrap();

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
        let execution_id = execution_id.to_owned();
        let jh = builder.spawn(run_task(task_path, execution_id)).unwrap();
        futures.push(jh);
    }


    let task_state_vec = join_all(futures).await;
    info!("job end {}", job_path_str);
    return task_state_vec;
}

async fn run_task<P: AsRef<Path>>(task_path: P,
                                  execution_id: String) -> TaskState {
    // mdc::insert("work_path", task_path.as_ref().to_str().unwrap());
    let app_context = flow::create_app_context(Box::new(PointRunnerDefault::new())).await;
    let rt = run_task0(task_path, execution_id.as_str(), app_context.as_ref()).await;
    match rt {
        Ok(ts) => ts,
        Err(e) => TaskState::Err(e)
    }
}

async fn run_task0<P: AsRef<Path>>(task_path: P,
                                   execution_id: &str,
                                   app_context: &dyn AppContext) -> Result<TaskState, Error> {
    let task_path = Path::new(task_path.as_ref());
    let task_work_path = task_path.join(format!("{}", execution_id));
    std::fs::create_dir(task_work_path.clone())?;

    info!("task start {}", task_path.to_str().unwrap());

    let flow_path = task_path.clone().join("flow.yml");

    let flow =port::load::flow::yml::load(&flow_path)?;
    let flow = Flow::new(flow.clone())?;

    //read
    let data_path = task_path.clone().join("data.csv");
    let mut data_reader = port::load::data::csv::from_path(data_path).await?;

    //write
    let result_path = task_work_path.clone().join("result.csv");
    let mut result_writer = port::report::csv::from_path(result_path).await?;
    port::report::csv::prepare(&mut result_writer, &flow).await?;

    let task_id = task_path.file_name().unwrap().to_str().unwrap();
    let mut total_task_state = TaskState::Ok(vec![]);
    let size_limit = 99999;
    loop{
        let data = port::load::data::csv::load(&mut data_reader, size_limit)?;
        let data_len = data.len();

        let task_assess = flow::run(app_context, flow.clone(), data, task_id).await;

        let _ = port::report::csv::report(&mut result_writer, task_assess.as_ref(), &flow).await?;

        match task_assess.state() {
            TaskState::Ok(_) => {},
            TaskState::Fail(_) => {
                total_task_state = TaskState::Fail(vec![]);
            }
            TaskState::Err(e) => {
                let result_path_old = task_work_path.clone().join("result.csv");
                let result_path_new = task_work_path.clone().join("result_E.csv");
                let _ = std::fs::rename(result_path_old, result_path_new);
                return Ok(TaskState::Err(e.clone()));
            }
        }

        if data_len < size_limit {
            break;
        }
    }

    let task_state_view = match total_task_state {
        TaskState::Ok(_) => "O",
        TaskState::Err(_) => "E",
        TaskState::Fail(_) => "F",
    };

    let result_path_old = task_work_path.clone().join("result.csv");
    let result_path_new = task_work_path.clone().join(format!("result_{}.csv", task_state_view));
    fs::rename(result_path_old, result_path_new).await.unwrap();

    info!("task end {}", task_path.to_str().unwrap());
    return Ok(total_task_state);
}