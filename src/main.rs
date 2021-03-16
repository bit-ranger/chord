use std::{env, fs};
use std::path::Path;
use std::time::SystemTime;

use futures::future::join_all;
use log::info;

use load::file;
use model::context::AppContextStruct;

use crate::model::context::{TaskResult, TaskError};

mod model;
mod flow;
mod point;
mod logger;
mod report;
mod load;

#[async_std::main]
async fn main() -> Result<(),usize> {
    let args: Vec<_> = env::args().collect();
    let mut opts = getopts::Options::new();
    opts.reqopt("j", "job", "job path", "job");
    opts.reqopt("l", "log", "log path", "log");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(_) => {
            println!("{}", opts.short_usage("runner"));
            return Err(1);
        }
    };

    let log_path = matches.opt_str("l").unwrap();
    logger::init(log::Level::Info,
                 log_path,
                 1,
                 2000000).unwrap();

    let duration = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH).unwrap();
    let execution_id = duration.as_millis().to_string();

    let job_path = matches.opt_str("j").unwrap();
    let job_path = Path::new(&job_path);
    if !job_path.is_dir(){
        panic!("job path is not a dir {}", job_path.to_str().unwrap());
    }

    // async_task::block_on(async {
        run_job(job_path, execution_id.as_str()).await;
    // });

    return Ok(());
}

async fn run_job<P: AsRef<Path>>(job_path: P, execution_id: &str) -> Vec<TaskResult>{
    let job_path_str = job_path.as_ref().to_str().unwrap();

    info!("running job {}", job_path_str);
    let children = fs::read_dir(job_path.as_ref()).unwrap();

    let mut futures = Vec::new();
    for task_dir in children{
        if task_dir.is_err(){
            continue;
        }
        let task_dir = task_dir.unwrap();
        if !task_dir.path().is_dir(){
            continue;
        }

        futures.push(
            run_task(job_path.as_ref().join(task_dir.path()), execution_id)
        );
    }

    let task_result_vec = join_all(futures).await;
    let task_status = task_result_vec.iter()
        .map(|r| r.as_ref().map_or_else(|e| Err(e.get_code()), |_| Ok(true)))
        .collect::<Vec<Result<bool, &str>>>();
    info!("finish job {}, {:?}", job_path_str, task_status);
    return task_result_vec;
}

async fn run_task<P: AsRef<Path>>(task_path: P, execution_id: &str) -> TaskResult{
    info!("running task {}", task_path.as_ref().to_str().unwrap());
    let task_path = Path::new(task_path.as_ref());
    let data_path = task_path.join("data.csv");
    let config_path = task_path.join("config.yml");
    let export_path = task_path.join(format!("export_{}.csv", execution_id));

    let data = match file::load_data(
        &data_path
    ) {
        Err(e) => {
            return TaskResult::Err(TaskError::new("000", format!("load data failure {}", e).as_str()));
        }
        Ok(vec) => {
            vec
        }
    };


    let config = match file::load_flow(
        &config_path
    ) {
        Err(e) => {
            return TaskResult::Err(TaskError::new("001", format!("load config failure {}", e).as_str()))
        }
        Ok(value) => {
            value
        }
    };

    let app_context = AppContextStruct::new();
    let task_result = flow::run(&app_context, config, data, task_path.file_name().unwrap().to_str().unwrap()).await;
    let _ = report::csv::export(&task_result, &export_path).await;
    info!("finish task {} >>> {}", task_path.to_str().unwrap(), task_result.is_ok());
    return task_result;
}
