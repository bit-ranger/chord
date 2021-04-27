use serde::{Serialize, Deserialize};
use chord_common::error::Error;
use validator::{Validate};
use std::time::SystemTime;
use chord_point::PointRunnerDefault;
use crate::biz;
use std::path::{PathBuf, Path};
use async_std::sync::Arc;
use chord_flow::AppContext;
use futures::executor::block_on;
use log::error;


#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct Req {
    #[validate(length(min = 1, max = 10))]
    name: String
}

pub struct Ctl {
    input: PathBuf,
    output: PathBuf,
    app_ctx: Arc<dyn AppContext>,
    pool: rayon::ThreadPool
}

static mut JOB_CTL: Option<Ctl> = Option::None;

impl Ctl {

    pub fn create_singleton(input: &str,
                            output: &str) -> &'static Ctl{
        unsafe {
            JOB_CTL = Some(Ctl::new(input, output));
            Ctl::get_singleton()
        }
    }

    pub fn get_singleton() -> &'static Ctl{
        unsafe {&JOB_CTL.as_ref().unwrap()}
    }

    pub fn new(input: &str,
               output: &str) -> Ctl {
        Ctl {
            input: Path::new(input).to_path_buf(),
            output: Path::new(output).to_path_buf(),
            app_ctx: block_on(chord_flow::create_app_context(Box::new(PointRunnerDefault::new()))),
            pool: rayon::ThreadPoolBuilder::new().build().unwrap(),
        }
    }

    pub async fn exec(&self, req: Req) -> Result<String, Error> {
        let exe_id = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_millis().to_string();
        let job_path = self.input.clone().join(&req.name);
        let work_path = self.output.clone().join(&req.name);

        let app_ctx_0 = self.app_ctx.clone();
        let exe_id_0 = exe_id.clone();
        self.pool.spawn(|| block_on(async move{
            if let Err(e) = async_std::fs::create_dir(work_path.clone()).await{
                error!("create_dir error {}", e)
            }
            let work_path = work_path.join(exe_id_0.as_str());
            if let Err(e) = async_std::fs::create_dir(work_path.clone()).await{
                error!("create_dir error {}", e)
            }
            let _task_state_vec = biz::job::run(job_path, work_path, exe_id_0, app_ctx_0).await;
        }));

        return Ok(exe_id);
    }
}