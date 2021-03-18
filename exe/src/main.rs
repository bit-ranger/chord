use std::{env, fs};
use std::path::Path;
use std::time::SystemTime;

use flow::{run_job, AppContextStruct};
use log::info;
use point::PointRunnerDefault;

mod logger;


#[async_std::main]
async fn main() -> Result<(),usize> {
    let args: Vec<_> = env::args().collect();
    let mut opts = getopts::Options::new();
    opts.reqopt("j", "job", "job path", "job");
    opts.reqopt("l", "log", "log path", "log");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(_) => {
            println!("{}", opts.short_usage("runner"));
            return Err(1);
        }
    };

    let log_path = matches.opt_str("l").unwrap();
    logger::init(log::Level::Info,
                 log_path,
                 1,
                 2000000).unwrap();

    let duration = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH).unwrap();
    let execution_id = duration.as_millis().to_string();

    let job_path = matches.opt_str("j").unwrap();
    let job_path = Path::new(&job_path);
    if !job_path.is_dir(){
        panic!("job path is not a dir {}", job_path.to_str().unwrap());
    }

    let app_context = AppContextStruct::new(Box::new(PointRunnerDefault::new()));
    // async_task::block_on(async {
        run_job(job_path, execution_id.as_str(), &app_context).await;
    // });

    return Ok(());
}

