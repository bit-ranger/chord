use std::env;
use std::path::Path;
use std::time::SystemTime;

use chord_common::{rerr};
use chord_common::error::Error;
use chord_common::task::TaskState;
use chord_point::PointRunnerDefault;
use itertools::Itertools;
use getopts::Matches;

mod logger;
mod job;




#[async_std::main]
async fn main() -> Result<(),Error> {
    let args: Vec<_> = env::args().collect();
    let mut opts = getopts::Options::new();
    opts.reqopt("j", "job", "job path", "job");
    opts.reqopt("w", "workspace", "workspace", "workspace");
    opts.optmulti("l", "level", "log level", "level");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(e) => {
            println!("{}", opts.short_usage("chord"));
            return rerr!("arg", e.to_string());
        }
    };

    let job_path = matches.opt_str("j").unwrap();
    let job_path = Path::new(&job_path);
    if !job_path.is_dir(){
        panic!("job is not a dir {}", job_path.to_str().unwrap());
    }

    let job_work_path = matches.opt_str("w").unwrap();
    let job_work_path = Path::new(&job_work_path);
    if !job_work_path.is_dir(){
        panic!("workspace is not a dir {}", job_work_path.to_str().unwrap());
    }

    let execution_id = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_millis().to_string();

    let job_work_path = job_work_path.join(execution_id.as_str());
    let job_work_path = Path::new(&job_work_path);
    std::fs::create_dir(job_work_path)?;

    let log_file_path = job_work_path.join("log.log");
    let log_handler = logger::init(target_level(matches), &log_file_path).await?;

    let app_ctx = chord_flow::create_app_context(Box::new(PointRunnerDefault::new())).await;
    let task_state_vec = job::run(job_path, job_work_path, execution_id, app_ctx).await;

    logger::terminal(log_handler).await?;

    let et = task_state_vec.iter().filter(|t| !t.is_ok()).last();
    return match et {
        Some(et) => {
            match et {
                TaskState::Ok(_) => Ok(()),
                TaskState::Err(e) => Err(e.clone()),
                TaskState::Fail(_) => rerr!("task", "fail")
            }
        },
        None => Ok(())
    };
}

fn target_level(matches: Matches) -> Vec<(String, String)>{
    let target_level: Vec<(String, String)> = matches.opt_strs("l")
        .into_iter()
        .map(|a| a.splitn(2, "=")
            .collect_tuple()
            .map(|(a, b)| (a.into(), b.into())).unwrap())
        .collect_vec();
    return target_level;
}