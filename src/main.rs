use std::{env};
use std::collections::BTreeMap;
use std::error::Error;
use std::fs::File;

use async_std::task as async_task;

use model::task::TaskContext;
use serde_json::Value;

mod model;
mod case;
mod point;
mod task;

fn load_data(path: &str) -> Result<Vec<BTreeMap<String,String>>, Box<dyn Error>> {
    let mut rdr = csv::Reader::from_path(path).unwrap();
    let mut hashmap_vec = Vec::new();
    for result in rdr.deserialize() {
        let record: BTreeMap<String, String> = result?;
        hashmap_vec.push(record);
    }
    Ok(hashmap_vec)
}


fn load_flow(path: &str) -> Result<Value, Box<dyn Error>>{
    let file = File::open(path).unwrap();

    let deserialized: Value= serde_yaml::from_reader(file)?;
    Ok(deserialized)
}

 fn main() {
    let args: Vec<_> = env::args().collect();
    let mut opts = getopts::Options::new();
    opts.reqopt("d", "data_file", "data file path", "data_file");
    opts.reqopt("f", "flow_file", "case file path", "flow_file");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(_) => {
            println!("{}", opts.short_usage("sqlgen"));
            return;
        }
    };

    let data_path = matches.opt_str("d").unwrap();

    let data = match load_data(
        &data_path
    ) {
        Err(e) => {
            panic!("{:?}", e)
        }
        Ok(vec) => {
            vec
        }
    };


    let flow_path = matches.opt_str("f").unwrap();

    let flow = match load_flow(
        &flow_path
    ) {
        Err(e) => {
            panic!("{:?}", e);
        }
        Ok(value) => {
            value
        }
    };

    let mut task_context = TaskContext::new(flow, data);
    async_task::block_on(async {
        let _ = task::run_task(&mut task_context).await;
    });
}
