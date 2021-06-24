use std::time::SystemTime;

use structopt::StructOpt;

use crate::conf::Config;
use async_std::fs::File;
use async_std::path::{Path, PathBuf};
use chord::rerr;
use chord::task::TaskState;
use chord::value::Value;
use chord::Error;
use chord_action::ActionFactoryDefault;
use futures::AsyncReadExt;

mod conf;
mod job;
mod logger;

#[async_std::main]
async fn main() -> Result<(), Error> {
    let opt = Opt::from_args();

    let input_dir = Path::new(&opt.input);
    if !input_dir.is_dir().await {
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

    let conf_data = load_conf(&opt.config).await?;
    let config = Config::new(conf_data);
    let log_handler = logger::init(config.log_level(), &log_file_path).await?;

    let flow_ctx = chord_flow::context_create(Box::new(
        ActionFactoryDefault::new(config.action_config().map(|c| c.clone())).await?,
    ))
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

async fn load_conf<P: AsRef<Path>>(path: P) -> Result<Value, Error> {
    let file = File::open(path).await;
    let mut file = match file {
        Err(_) => return Ok(Value::Null),
        Ok(r) => r,
    };
    let mut content = String::new();
    file.read_to_string(&mut content).await?;

    let deserialized: Result<Value, serde_yaml::Error> = serde_yaml::from_str(content.as_str());
    return match deserialized {
        Err(e) => return rerr!("yaml", format!("{:?}", e)),
        Ok(r) => Ok(r),
    };
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

    /// config file path
    #[structopt(
        short,
        long,
        parse(from_os_str),
        default_value = "/data/chord/conf/application.yml"
    )]
    config: PathBuf,
}
