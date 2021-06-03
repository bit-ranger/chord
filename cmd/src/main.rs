use std::path::Path;
use std::time::SystemTime;

use chord_common::error::Error;
use chord_common::rerr;
use chord_common::task::TaskState;
use chord_common::value::Map;
use chord_step::StepRunnerFactoryDefault;
use itertools::Itertools;
use std::path::PathBuf;
use structopt::StructOpt;

mod job;
mod logger;

#[async_std::main]
async fn main() -> Result<(), Error> {
    let opt = Opt::from_args();

    let input_dir = Path::new(&opt.input);
    if !input_dir.is_dir() {
        panic!("input is not a dir {}", input_dir.to_str().unwrap());
    }

    let exec_id = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_millis()
        .to_string();

    let output_dir = Path::new(&opt.output).join(exec_id.as_str());
    let output_dir = output_dir.as_path();
    async_std::fs::create_dir_all(output_dir).await?;

    let log_file_path = output_dir.join("log.log");
    let log_handler = logger::init(target_level(&opt.level), &log_file_path).await?;

    let flow_ctx =
        chord_flow::create_context(Box::new(StepRunnerFactoryDefault::new(Map::new()).await?))
            .await;
    let task_state_vec = job::run(input_dir, output_dir, exec_id, flow_ctx).await;

    logger::terminal(log_handler).await?;

    let et = task_state_vec.iter().filter(|t| !t.is_ok()).last();
    return match et {
        Some(et) => match et {
            TaskState::Ok => Ok(()),
            TaskState::Err(e) => Err(e.clone()),
            TaskState::Fail => rerr!("task", "fail"),
        },
        None => Ok(()),
    };
}

fn target_level(level: &Vec<String>) -> Vec<(String, String)> {
    let target_level = level
        .iter()
        .map(|a| {
            a.splitn(2, "=")
                .collect_tuple()
                .map(|(a, b)| (a.into(), b.into()))
                .unwrap()
        })
        .collect_vec();
    return target_level;
}

#[derive(StructOpt, Debug)]
#[structopt(name = "chord")]
struct Opt {
    /// input dir
    #[structopt(short, long, parse(from_os_str))]
    input: PathBuf,

    /// output dir
    #[structopt(short, long, parse(from_os_str))]
    output: PathBuf,

    /// log level
    #[structopt(short, long)]
    level: Vec<String>,
}
