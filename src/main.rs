use std::env;
use async_std::task as async_task;


use crate::model::app::AppContextStruct;

mod model;
mod loader;
mod flow;
mod point;

fn main() {
    let args: Vec<_> = env::args().collect();
    let mut opts = getopts::Options::new();
    opts.reqopt("d", "data", "data file path", "data_file");
    opts.reqopt("c", "config", "config file path", "config_file");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(_) => {
            println!("{}", opts.short_usage("sqlgen"));
            return;
        }
    };

    let data_path = matches.opt_str("d").unwrap();

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


    let config_path = matches.opt_str("c").unwrap();

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

    let app_context = AppContextStruct::new();

    async_task::block_on(async {
        let _ = flow::run(&app_context, config, data ).await;
    });
}
