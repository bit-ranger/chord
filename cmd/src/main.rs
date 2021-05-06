use std::env;
use std::path::Path;
use std::time::SystemTime;

use chord_common::{rerr};
use chord_common::error::Error;
use chord_common::task::TaskState;
use chord_point::PointRunnerFactoryDefault;
use itertools::Itertools;
use getopts::Matches;

mod logger;
mod job;




#[async_std::main]
async fn main() -> Result<(),Error> {
    let args: Vec<_> = env::args().collect();
    let mut opts = getopts::Options::new();
    opts.reqopt("i", "input", "input path", "input");
    opts.reqopt("o", "output", "output path", "output");
    opts.optmulti("l", "level", "log level", "level");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(e) => {
            println!("{}", opts.short_usage("chord"));
            return rerr!("arg", e.to_string());
        }
    };

    let input_path = matches.opt_str("i").unwrap();
    let input_dir = Path::new(&input_path);
    if !input_dir.is_dir(){
        panic!("input is not a dir {}", input_dir.to_str().unwrap());
    }

    let execution_id = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_millis().to_string();

    let output_path = matches.opt_str("o").unwrap();
    let output_dir = Path::new(&output_path).join(execution_id.as_str());
    let output_dir = output_dir.as_path();
    async_std::fs::create_dir_all(output_dir).await?;

    let log_file_path = output_dir.join("log.log");
    let log_handler = logger::init(target_level(matches), &log_file_path).await?;

    let flow_ctx = chord_flow::create_context(Box::new(PointRunnerFactoryDefault::new().await?)).await;
    let task_state_vec = job::run(input_dir, output_dir, execution_id, flow_ctx).await;

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