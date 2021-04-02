use std::env;
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::SystemTime;

use common::{cause, err};
use common::error::Error;
use common::task::TaskState;
use point::PointRunnerDefault;
use itertools::Itertools;

mod logger;
mod job;




#[async_std::main]
async fn main() -> Result<(),Error> {
    let args: Vec<_> = env::args().collect();
    let mut opts = getopts::Options::new();
    opts.reqopt("j", "job", "job path", "job");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(e) => {
            println!("{}", opts.short_usage("chord"));
            return err!("arg", e.to_string());
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

    let log_file_path = Path::new(job_path).join(format!("log_{}.log", execution_id));
    let log_enable = Arc::new(AtomicBool::new(true));
    let target_level: Vec<(String, String)> = env::args()
        .into_iter()
        .filter(|a| a.starts_with("log.level."))
        .map(|a| a[9..].splitn(2, "=")
            .collect_tuple()
            .map(|(a, b)| (a.into(), b.into())).unwrap())
        .collect_vec();
    let log_handler = logger::init(
        target_level,
        &log_file_path,
        log_enable.clone()).await?;

    let app_ctx = flow::create_app_context(Box::new(PointRunnerDefault::new())).await;
    let task_state_vec = job::run(job_path, execution_id.as_str(), app_ctx).await;

    log_enable.store(false, Ordering::SeqCst);
    let _ = log_handler.join();

    let et = task_state_vec.iter().filter(|t| !t.is_ok()).last();
    return match et {
        Some(et) => {
            match et {
                TaskState::Ok(_) => Ok(()),
                TaskState::Err(e) => cause!("task", e.to_string(), e.clone()),
                TaskState::Fail(_) => err!("task", "fail")
            }
        },
        None => Ok(())
    };
}