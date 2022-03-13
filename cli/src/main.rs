use std::fmt::{Debug, Display, Formatter};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use dirs;
use structopt::StructOpt;

use chord_action::FactoryComposite;
use chord_core::future::path::is_dir;
use chord_core::task::TaskState;
use chord_core::value::Value;
use chord_input::load::DefaultJobLoader;
use chord_output::report::DefaultJobReporter;

use crate::conf::Config;
use crate::job::dir_is_task_path;
use crate::RunError::{InputNotDir, Logger, TaskErr, TaskFail};

mod conf;
mod job;
mod logger;

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
        #[structopt(short, long, parse(from_os_str), default_value = ".")]
        input: PathBuf,

        /// config file path
        #[structopt(short, long, parse(from_os_str))]
        config: Option<PathBuf>,

        /// print verbose info
        #[structopt(long)]
        verbose: bool,
    },
}

#[derive(thiserror::Error)]
enum RunError {
    #[error("input is not a dir: {0}")]
    InputNotDir(String),

    #[error("config error:\n{0}")]
    Config(chord_input::conf::Error),

    #[error("report error:\n{0}")]
    Report(chord_core::output::Error),

    #[error("action factory error:\n{0}")]
    ActionFactory(chord_core::action::Error),

    #[error("log error:\n{0}")]
    Logger(logger::Error),

    #[error("job error:\n{0}")]
    JobErr(job::Error),

    #[error("task fail: `{0}`\n{1}")]
    TaskFail(String, String),

    #[error("task error: `{0}`\n{1}")]
    TaskErr(String, String),
}

#[chord_core::future::main]
async fn main() -> Result<(), RunError> {
    let opt = Chord::from_args();
    match opt {
        Chord::Run {
            job_name,
            exec_id,
            input,
            config,
            verbose,
        } => run(job_name, exec_id, input, config, verbose).await,
    }
}

async fn run(
    job_name: String,
    exec_id: String,
    input: PathBuf,
    config: Option<PathBuf>,
    verbose: bool,
) -> Result<(), RunError> {
    let input_dir = Path::new(&input);
    if !is_dir(input_dir).await {
        return Err(InputNotDir(input_dir.to_str().unwrap().to_string()));
    }

    let exec_id: String = exec_id.clone();
    let job_name = job_name.clone();

    let conf_dir_path = config
        .clone()
        .map(|p| PathBuf::from(p))
        .unwrap_or_else(|| PathBuf::from(dirs::home_dir().unwrap().join(".chord").join("conf")));

    let conf_data = if chord_input::conf::exists(conf_dir_path.as_path(), "cmd").await {
        chord_input::conf::load(conf_dir_path.as_path(), "cmd")
            .await
            .map_err(|e| RunError::Config(e))?
    } else {
        Value::Null
    };

    let config = Config::new(conf_data);
    if verbose {
        println!("config loaded: {}", config);
    }

    let log = logger::Log::new(config.log_level())
        .await
        .map_err(|e| Logger(e))?;

    let path_is_task = dir_is_task_path(input_dir.to_path_buf()).await;
    let job_loader = DefaultJobLoader::new(config.loader(), input_dir.clone(), path_is_task)
        .await
        .map_err(|e| RunError::Report(e))?;
    let job_loader = Arc::new(job_loader);

    let job_reporter =
        DefaultJobReporter::new(config.reporter(), job_name.as_str(), exec_id.as_str())
            .await
            .map_err(|e| RunError::Report(e))?;
    let job_reporter = Arc::new(job_reporter);

    let app = chord_flow::app_create(
        FactoryComposite::new(config.action().map(|c| c.clone()))
            .await
            .map_err(|e| RunError::ActionFactory(e))?
            .into(),
    )
    .await;
    let task_state_vec = job::run(
        app,
        job_loader,
        job_reporter,
        exec_id.clone(),
        input_dir,
        path_is_task,
    )
    .await
    .map_err(|e| RunError::JobErr(e))?;
    log.drop().await;
    let et = task_state_vec.iter().filter(|t| !t.state().is_ok()).nth(0);
    return match et {
        Some(et) => match et.state() {
            TaskState::Ok => Ok(()),
            TaskState::Err(e) => Err(TaskErr(et.id().task().to_string(), e.to_string())),
            TaskState::Fail(c) => Err(TaskFail(et.id().task().to_string(), c.to_string())),
        },
        None => Ok(()),
    };
}

impl Debug for RunError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self, f)
    }
}
