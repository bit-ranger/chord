use serde::{Serialize, Deserialize};
use chord_common::error::Error;
use validator::{Validate};
use std::time::SystemTime;
use chord_point::PointRunnerDefault;
use crate::service;
use std::path::{PathBuf, Path};
use async_std::sync::Arc;
use chord_flow::AppContext;
use futures::executor::block_on;


#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct Req {
    #[validate(length(min = 1, max = 10))]
    name: String
}

pub struct Ctl {
    job_dir: PathBuf,
    work_dir: PathBuf,
    app_ctx: Arc<dyn AppContext>,
    pool: rayon::ThreadPool
}

impl Ctl {

    pub  fn new(job_dir: String,
               work_dir: String) -> Ctl {
        Ctl {
            job_dir: Path::new(job_dir.as_str()).to_path_buf(),
            work_dir: Path::new(work_dir.as_str()).to_path_buf(),
            app_ctx: block_on(chord_flow::create_app_context(Box::new(PointRunnerDefault::new()))),
            pool: rayon::ThreadPoolBuilder::new().build().unwrap()
        }
    }


    pub async fn exec(&self, req: Req) -> Result<String, Error> {
        let exe_id = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_millis().to_string();
        let job_path = self.job_dir.clone().join(&req.name);
        let work_path = self.work_dir.clone().join(&req.name).join(exe_id.as_str());

        let app_ctx_0 = self.app_ctx.clone();
        let exe_id_0 = exe_id.clone();
        self.pool.spawn(|| block_on(async {
            let _ = async_std::fs::create_dir(work_path.clone()).await;
            let _task_state_vec = service::job::run(job_path, work_path, exe_id_0, app_ctx_0).await;
        }));

        return Ok(exe_id);
    }
}
