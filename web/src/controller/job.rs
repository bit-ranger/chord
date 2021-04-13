use tide::prelude::*;
use chord_common::error::Error;
use validator::{Validate};
use std::time::SystemTime;
use async_std::path::Path;
use chord_point::PointRunnerDefault;
use crate::service;


#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct Job {
    #[validate(length(min = 1, max = 10))]
    name: String
}

pub async fn exec(job: Job) -> Result<String, Error> {
    let execution_id = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_millis().to_string();
    let app_ctx = chord_flow::create_app_context(Box::new(PointRunnerDefault::new())).await;
    let job_path = Path::new("/data/chord").join(&job.name);
    let _task_state_vec = service::job::run(job_path, execution_id.as_str(), app_ctx).await;
    return Ok(execution_id);
}