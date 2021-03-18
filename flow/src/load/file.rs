use std::collections::BTreeMap;
use std::fs::File;
use std::path::Path;

use serde_json::Value;

use common::error::Error;
use common::err;

pub fn load_data<P: AsRef<Path>>(path: P) -> Result<Vec<BTreeMap<String, String>>, Error> {
    let rdr = csv::Reader::from_path(path);
    let mut rdr = match rdr {
        Err(e) => return err!("csv", format!("{:?}", e.kind()).as_str()),
        Ok(r) => r
    };
    let mut hashmap_vec = Vec::new();
    for result in rdr.deserialize() {
        let result = match result  {
            Err(e)  => return err!("csv", format!("{:?}", e).as_str()),
            Ok(r) => r
        };

        let record: BTreeMap<String, String> = result;

        hashmap_vec.push(record);
    }
    Ok(hashmap_vec)
}


pub fn load_flow<P: AsRef<Path>>(path: P) -> Result<Value, Error> {
    let file = File::open(path);
    let file = match file {
        Err(e) => return err!("yaml", format!("{:?}", e).as_str()),
        Ok(r) => r
    };

    let deserialized:Result<Value, serde_yaml::Error> = serde_yaml::from_reader(file);
    return match deserialized {
        Err(e) => return err!("yaml", format!("{:?}", e).as_str()),
        Ok(r) => Ok(r)
    };
}