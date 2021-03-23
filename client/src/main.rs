use std::{env, fs};
use std::path::Path;
use std::time::SystemTime;

use futures::future::join_all;
use log::info;

use common::err;
use common::error::Error;
use common::flow::Flow;
use common::task::TaskState;
use flow::AppContext;
use point::PointRunnerDefault;

mod logger;

#[async_std::main]
async fn main() -> Result<(),Error> {
    let args: Vec<_> = env::args().collect();
    let mut opts = getopts::Options::new();
    opts.reqopt("j", "job", "job path", "job");
    opts.reqopt("l", "log", "log path", "log");
    opts.optopt("p", "print", "console print", "true/false");
    opts.optopt("t", "target", "long target", ".*");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(e) => {
            println!("{}", opts.short_usage("chord"));
            return err!("arg", e.to_string().as_str());
        }
    };

    logger::init(matches.opt_get_default("t", String::from(".*")).unwrap(),
                 matches.opt_str("l").unwrap(),
                 1,
                 2000000,
                 matches.opt_get_default("p", false).unwrap()
    ).unwrap();

    let duration = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH).unwrap();
    let execution_id = duration.as_millis().to_string();

    let job_path = matches.opt_str("j").unwrap();
    let job_path = Path::new(&job_path);
    if !job_path.is_dir(){
        panic!("job path is not a dir {}", job_path.to_str().unwrap());
    }

    let app_context = flow::create_app_context(Box::new(PointRunnerDefault::new())).await;
    let task_state_vec = run_job(job_path, execution_id.as_str(), app_context.as_ref()).await;

    let et = task_state_vec.iter().filter(|t| !t.is_ok()).last();

    return match et {
        Some(et) => {
            match et {
                TaskState::Ok(_) => Ok(()),
                TaskState::Err(e) => err!("task", e.to_string().as_str()),
                TaskState::Fail(_) => err!("task", "fail")
            }

        },
        None => Ok(())
    };
}


pub async fn run_job<P: AsRef<Path>>(job_path: P,
                                     execution_id: &str,
                                     app_context: &dyn AppContext) -> Vec<TaskState>{
    let job_path_str = job_path.as_ref().to_str().unwrap();

    info!("running job {}", job_path_str);
    let children = fs::read_dir(job_path.as_ref()).unwrap();

    let mut futures = Vec::new();
    for task_dir in children{
        if task_dir.is_err(){
            continue;
        }
        let task_dir = task_dir.unwrap();
        if !task_dir.path().is_dir(){
            continue;
        }

        futures.push(
            run_task(job_path.as_ref().join(task_dir.path()), execution_id, app_context)
        );
    }

    let task_state_vec = join_all(futures).await;
    // info!("finish job {}, {}", job_path_str, task_state_vec);
    return task_state_vec;
}

async fn run_task<P: AsRef<Path>>(task_path: P,
                                  execution_id: &str,
                                  app_context: &dyn AppContext) -> TaskState {
    let rt = run_task0(task_path, execution_id, app_context).await;
    match rt {
        Ok(ts) => ts,
        Err(e) => TaskState::Err(e)
    }
}

async fn run_task0<P: AsRef<Path>>(task_path: P,
                                   execution_id: &str,
                                   app_context: &dyn AppContext) -> Result<TaskState, Error> {
    info!("running task {}", task_path.as_ref().to_str().unwrap());
    let task_path = Path::new(task_path.as_ref());
    let flow_path = task_path.clone().join("flow.yml");

    let flow =port::load::flow::yml::load(&flow_path)?;
    let flow = Flow::new(flow.clone())?;

    let report_dir_path = task_path.join(format!("{}", execution_id));
    std::fs::create_dir(report_dir_path.clone())?;

    //read
    let data_path = task_path.clone().join("data.csv");
    let mut data_reader = port::load::data::csv::from_path(data_path).await?;

    //write
    let result_path = report_dir_path.clone().join("result.csv");
    let mut result_writer = port::report::csv::from_path(result_path).await?;
    port::report::csv::prepare(&mut result_writer, &flow).await?;

    let task_id = task_path.file_name().unwrap().to_str().unwrap();
    let mut total_task_state = TaskState::Ok(vec![]);
    let size_limit = 99999;
    loop{
        let data = port::load::data::csv::load(&mut data_reader, size_limit)?;
        let data_len = data.len();

        let task_assess = flow::run(app_context, flow.clone(), data, task_id).await;

        let _ = port::report::csv::report(&mut result_writer, task_assess.as_ref(), &flow).await?;

        match task_assess.state() {
            TaskState::Ok(_) => {},
            TaskState::Fail(_) => {
                total_task_state = TaskState::Fail(vec![]);
            }
            TaskState::Err(e) => {
                let result_path_old = report_dir_path.clone().join("result.csv");
                let result_path_new = report_dir_path.clone().join("result_E.csv");
                let _ = std::fs::rename(result_path_old, result_path_new);
                return Ok(TaskState::Err(e.clone()));
            }
        }

        if data_len < size_limit {
            break;
        }
    }

    let task_state_view = match total_task_state {
        TaskState::Ok(_) => "O",
        TaskState::Err(_) => "E",
        TaskState::Fail(_) => "F",
    };

    let result_path_old = report_dir_path.clone().join("result.csv");
    let result_path_new = report_dir_path.clone().join(format!("result_{}.csv", task_state_view));
    let _ = std::fs::rename(result_path_old, result_path_new);

    info!("finish task {}", task_path.to_str().unwrap());
    return Ok(total_task_state);
}