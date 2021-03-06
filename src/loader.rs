use std::collections::BTreeMap;
use std::error::Error;
use std::fs::File;

use serde_json::Value;

pub fn load_data(path: &str) -> Result<Vec<BTreeMap<String, String>>, Box<dyn Error>> {
    let mut rdr = csv::Reader::from_path(path).unwrap();
    let mut hashmap_vec = Vec::new();
    for result in rdr.deserialize() {
        let record: BTreeMap<String, String> = result?;
        hashmap_vec.push(record);
    }
    Ok(hashmap_vec)
}


pub fn load_flow(path: &str) -> Result<Value, Box<dyn Error>> {
    let file = File::open(path).unwrap();

    let deserialized: Value = serde_yaml::from_reader(file)?;
    Ok(deserialized)
}
