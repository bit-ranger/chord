use std::time::SystemTime;

use async_std::fs::File;
use async_std::path::{Path, PathBuf};
use futures::AsyncReadExt;
use structopt::StructOpt;

use chord::err;
use chord::task::TaskState;
use chord::value::Value;
use chord::Error;
use chord_action::FactoryComposite;
use chord_input::load::flow::yml::YmlFlowParser;

use crate::conf::Config;

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

    let exec_id: &Option<String> = &opt.exec_id;
    let exec_id: String = match exec_id {
        Some(id) => id.clone(),
        None => (SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_millis()
            - 1622476800000)
            .to_string(),
    };

    let conf_data = load_conf(&opt.config).await?;
    let config = Config::new(conf_data);

    let log_file_path = Path::new(config.log_path());
    let log_handler = logger::init(config.log_level(), &log_file_path).await?;

    let flow_ctx = chord_flow::context_create(
        Box::new(FactoryComposite::new(config.action().map(|c| c.clone())).await?),
        Box::new(YmlFlowParser::new()),
    )
    .await;
    let task_state_vec = job::run(
        opt.job_name.clone(),
        input_dir,
        opt.task,
        exec_id,
        flow_ctx,
        &config,
    )
    .await?;
    logger::terminal(log_handler).await?;
    let et = task_state_vec.iter().filter(|t| !t.is_ok()).last();
    return match et {
        Some(et) => match et {
            TaskState::Ok => Ok(()),
            TaskState::Err(e) => Err(e.clone()),
            TaskState::Fail => Err(err!("task", "fail")),
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
        Err(e) => return Err(err!("conf", format!("{:?}", e))),
        Ok(r) => Ok(r),
    };
}

#[derive(StructOpt, Debug)]
#[structopt(name = "chord")]
struct Opt {
    /// job name
    #[structopt(short, long, default_value = "chord_cmd")]
    job_name: String,

    /// exec id
    #[structopt(short, long)]
    exec_id: Option<String>,

    /// input dir
    #[structopt(short, long, parse(from_os_str))]
    input: PathBuf,

    /// task list
    #[structopt(short, long)]
    task: Option<Vec<String>>,

    /// config file path
    #[structopt(
        short,
        long,
        parse(from_os_str),
        default_value = "/data/chord/conf/cmd.yml"
    )]
    config: PathBuf,
}
