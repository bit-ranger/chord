use tide::prelude::*;
use chord_common::error::Error;
use validator::{Validate};
use std::time::SystemTime;
use chord_point::PointRunnerDefault;
use crate::service;
use std::path::{PathBuf, Path};


#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct Req {
    #[validate(length(min = 1, max = 10))]
    name: String
}

pub struct Ctl {
    job_dir: PathBuf,
    work_dir: PathBuf
}

impl Ctl {

    pub fn new(job_dir: String,
               work_dir: String) -> Ctl {
        Ctl {
            job_dir: Path::new(job_dir.as_str()).to_path_buf(),
            work_dir: Path::new(work_dir.as_str()).to_path_buf()
        }
    }

    pub async fn exec(&self, req: Req) -> Result<String, Error> {
        let execution_id = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_millis().to_string();
        let app_ctx = chord_flow::create_app_context(Box::new(PointRunnerDefault::new())).await;
        let job_path = self.job_dir.clone().join(&req.name);
        let work_path = self.work_dir.clone().join(&req.name);
        let _task_state_vec = service::job::run(job_path, work_path, execution_id.clone(), app_ctx).await;
        return Ok(execution_id);
    }
}
