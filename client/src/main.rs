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

    let log_path = matches.opt_str("l").unwrap();
    logger::init(log::Level::Info,
                 matches.opt_get_default("t", String::from(".*")).unwrap(),
                 log_path,
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

    let app_context = flow::mk_app_context(Box::new(PointRunnerDefault::new())).await;
    let task_state_vec = run_job(job_path, execution_id.as_str(), app_context.as_ref()).await;

    let et = task_state_vec.iter().filter(|t| t.is_err()).last();
    return match et {
        Some(et) => {
            match et {
                Ok(_) => Ok(()),
                Err(e) => err!("task", e.as_str())
            }

        },
        None => Ok(())
    };
}


pub async fn run_job<P: AsRef<Path>>(job_path: P,
                                     execution_id: &str,
                                     app_context: &dyn AppContext) -> Vec<Result<bool,String>>{
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

    let task_result_vec = join_all(futures).await;
    let task_status = task_result_vec.iter()
        .map(|r|
                r.as_ref().map_or_else(|e| Err(String::from(e.code())),
                                       |_| Ok(true)))
        .collect::<Vec<Result<bool, String>>>();
    info!("finish job {}, {:?}", job_path_str, task_status);
    return task_status;
}

async fn run_task<P: AsRef<Path>>(task_path: P,
                                  execution_id: &str,
                                  app_context: &dyn AppContext) -> Result<TaskState,Error> {
    info!("running task {}", task_path.as_ref().to_str().unwrap());
    let task_path = Path::new(task_path.as_ref());
    let flow_path = task_path.clone().join("flow.yml");

    let flow = match port::load::flow::yml::load(&flow_path) {
        Err(e) => {
            return err!("001", format!("load flow failure {}", e).as_str())
        }
        Ok(value) => {
            value
        }
    };
    let flow = Flow::new(flow.clone());

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
    let mut task_state = TaskState::Ok;
    let size_limit = 99999;
    loop{
        let data = match port::load::data::csv::load(&mut data_reader, size_limit) {
            Err(e) => {
                return err!("000", format!("load data failure {}", e).as_str());
            }
            Ok(vec) => {
                vec
            }
        };
        let data_len = data.len();

        let task_result = flow::run(app_context, flow.clone(), data, task_id).await;

        let _ = port::report::csv::report(&mut result_writer, &task_result, &flow).await?;

        match task_result {
            Ok(tr) => {
                if task_state.is_ok(){
                    task_state = tr.state().clone();
                }
            },
            Err(e) => {
                let result_path_old = report_dir_path.clone().join("result.csv");
                let result_path_new = report_dir_path.clone().join("result_E.csv");
                let _ = std::fs::rename(result_path_old, result_path_new);
                return Err(e);
            }
        }

        if data_len < size_limit {
            break;
        }
    }

    let result_state = match task_state {
        TaskState::Ok => "O",
        _ => "F"
    };

    let result_path_old = report_dir_path.clone().join("result.csv");
    let result_path_new = report_dir_path.clone().join(format!("result_{}.csv", result_state));
    let _ = std::fs::rename(result_path_old, result_path_new);

    info!("finish task {}", task_path.to_str().unwrap());
    return Ok(task_state);
}