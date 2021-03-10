use std::env;
use async_std::task as async_task;


use model::context::AppContextStruct;
use log::{info};

mod model;
mod loader;
mod flow;
mod point;
mod logger;

fn main() {
    let args: Vec<_> = env::args().collect();
    let mut opts = getopts::Options::new();
    opts.reqopt("d", "data", "data file path", "data");
    opts.reqopt("c", "config", "config file path", "config");
    opts.reqopt("l", "log", "log file path", "log");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(_) => {
            println!("{}", opts.short_usage("runner"));
            return;
        }
    };

    let data_path = matches.opt_str("d").unwrap();
    let log_path = matches.opt_str("l").unwrap();
    let config_path = matches.opt_str("c").unwrap();

    let data = match loader::load_data(
        &data_path
    ) {
        Err(e) => {
            panic!("{:?}", e)
        }
        Ok(vec) => {
            vec
        }
    };


    let config = match loader::load_flow(
        &config_path
    ) {
        Err(e) => {
            panic!("{:?}", e);
        }
        Ok(value) => {
            value
        }
    };

    logger::init(log::Level::Info,
                 log_path,
                 1,
                 2000000).unwrap();

    let app_context = AppContextStruct::new();

    async_task::block_on(async {
        let task_result = flow::run(&app_context, config, data ).await;
        info!("task result {:?}", task_result);
    });
}
