use async_std::fs::File;
use async_std::path::{Path, PathBuf};
use futures::AsyncReadExt;
use structopt::StructOpt;

use chord::err;
use chord::task::TaskState;
use chord::value::Value;
use chord::Error;
use chord_action::FactoryComposite;

use crate::conf::Config;
use async_std::sync::Arc;
use chord_output::report::ReportFactory;
use dirs;

mod conf;
mod job;
mod logger;

#[async_std::main]
async fn main() -> Result<(), Error> {
    let opt = Chord::from_args();
    match opt {
        Chord::Run {
            job_name,
            exec_id,
            input,
            task,
            config,
            verbose,
        } => run(job_name, exec_id, input, task, config, verbose).await?,
    }
    Ok(())
}

async fn run(
    job_name: String,
    exec_id: String,
    input: PathBuf,
    task: Option<Vec<String>>,
    config: Option<PathBuf>,
    verbose: bool,
) -> Result<(), Error> {
    let input_dir = Path::new(&input);
    if !input_dir.is_dir().await {
        panic!("input is not a dir {}", input_dir.to_str().unwrap());
    }

    let exec_id: String = exec_id.clone();
    let job_name = job_name.clone();

    let conf_path = config.clone().map(|p| PathBuf::from(p)).unwrap_or_else(|| {
        PathBuf::from(
            dirs::home_dir()
                .unwrap()
                .join(".chord")
                .join("conf")
                .join("cmd.yml"),
        )
    });
    let conf_data = load_conf(conf_path).await?;
    let config = Config::new(conf_data);
    if verbose {
        println!("config loaded: {}", config);
    }

    let report_factory =
        ReportFactory::new(config.report(), job_name.as_str(), exec_id.as_str()).await?;
    let report_factory = Arc::new(report_factory);

    let log_file_path = config
        .log_dir()
        .join(job_name.clone())
        .join(exec_id.clone())
        .join("cmd.log");
    let log_handler = logger::init(config.log_level(), log_file_path.as_path()).await?;

    let flow_ctx = chord_flow::context_create(Box::new(
        FactoryComposite::new(config.action().map(|c| c.clone())).await?,
    ))
    .await;
    let task_state_vec = job::run(
        report_factory,
        input_dir,
        task.clone(),
        exec_id.clone(),
        flow_ctx,
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

#[derive(StructOpt)]
#[structopt(name = "chord")]
enum Chord {
    Run {
        /// job name
        #[structopt(short, long, default_value = "chord_cmd")]
        job_name: String,

        /// exec id
        #[structopt(short, long, default_value = "1")]
        exec_id: String,

        /// input dir
        #[structopt(short, long, parse(from_os_str))]
        input: PathBuf,

        /// task list
        #[structopt(short, long)]
        task: Option<Vec<String>>,

        /// config file path
        #[structopt(short, long, parse(from_os_str))]
        config: Option<PathBuf>,

        #[structopt(long)]
        verbose: bool,
    },
}
