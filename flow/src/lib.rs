use std::fs;
use std::path::Path;
use log::info;
use common::error::Error;

use crate::load::file;
use crate::model::app::{AppContext};
use crate::model::task::TaskResult;
use common::point::PointRunner;
use futures::future::join_all;

mod model;
pub mod flow;
pub mod report;
pub mod load;

struct ErrorWrapper(Error);

pub use crate::model::app::AppContextStruct;

pub async fn run_job<P: AsRef<Path>>(job_path: P, execution_id: &str, app_context: &dyn AppContext) -> Vec<TaskResult>{
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
            run_task(job_path.as_ref().join(task_dir.path()), execution_id, app_context)
        );
    }

    let task_result_vec = join_all(futures).await;
    let task_status = task_result_vec.iter()
        .map(|r| r.as_ref().map_or_else(|e| Err(e.get_code()), |_| Ok(true)))
        .collect::<Vec<Result<bool, &str>>>();
    info!("finish job {}, {:?}", job_path_str, task_status);
    return task_result_vec;
}

async fn run_task<P: AsRef<Path>>(task_path: P, execution_id: &str, app_context: &dyn AppContext) -> TaskResult {
    info!("running task {}", task_path.as_ref().to_str().unwrap());
    let task_path = Path::new(task_path.as_ref());
    let data_path = task_path.join("data.csv");
    let config_path = task_path.join("config.yml");
    let export_path = task_path.join(format!("export_{}.csv", execution_id));

    let data = match file::load_data(
        &data_path
    ) {
        Err(e) => {
            return Err(Error::new("000", format!("load data failure {}", e).as_str()));
        }
        Ok(vec) => {
            vec
        }
    };


    let config = match file::load_flow(
        &config_path
    ) {
        Err(e) => {
            return Err(Error::new("001", format!("load config failure {}", e).as_str()))
        }
        Ok(value) => {
            value
        }
    };

    let task_result = flow::run(app_context, config, data, task_path.file_name().unwrap().to_str().unwrap()).await;
    let _ = report::csv::export(&task_result, &export_path).await;
    info!("finish task {} >>> {}", task_path.to_str().unwrap(), task_result.is_ok());
    return task_result;
}