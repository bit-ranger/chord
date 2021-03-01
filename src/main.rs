use std::env;
use std::error::Error;
use std::process;
use std::fs::File;
use std::collections::BTreeMap;
use serde_yaml::Value;


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
    let mut file = File::open(path).unwrap();

    let deserialized: Value= serde_yaml::from_reader(file)?;
    Ok(deserialized)
}



fn main() {
    let args: Vec<_> = env::args().collect();
    let mut opts = getopts::Options::new();
    opts.reqopt("d", "data_file", "data file path", "data_file");
    opts.reqopt("f", "flow_file", "flow file path", "flow_file");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(_) => {
            println!("{}", opts.short_usage("sqlgen"));
            return;
        }
    };

    let data_path = matches.opt_str("d").unwrap();

    match load_data(
        &data_path) {
        Err(_e) => {
            process::exit(1);
        },
        Ok(vec) => {
            for data in vec {
                println!("{:?}", data)
            }
        }
    }


    let flow_path = matches.opt_str("f").unwrap();

    match load_flow(
        &flow_path) {
        Err(_e) => {
            println!("{:?}", _e);
            process::exit(1);
        },
        Ok(value) => {
            println!("{:?}", value);
        }
    }
}