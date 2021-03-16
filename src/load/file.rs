use std::collections::BTreeMap;
use std::fs::File;
use std::path::Path;

use serde_json::Value;

use crate::model::error::Error;
use crate::model::context::BasicError;

pub fn load_data<P: AsRef<Path>>(path: P) -> Result<Vec<BTreeMap<String, String>>, BasicError> {
    let mut rdr = csv::Reader::from_path(path)?;
    let mut hashmap_vec = Vec::new();
    for result in rdr.deserialize() {
        let record: BTreeMap<String, String> = result?;
        hashmap_vec.push(record);
    }
    Ok(hashmap_vec)
}


pub fn load_flow<P: AsRef<Path>>(path: P) -> Result<Value, BasicError> {
    let file = File::open(path)?;

    let deserialized: Value = serde_yaml::from_reader(file)?;
    Ok(deserialized)
}


impl From<csv::Error> for BasicError {
    fn from(err: csv::Error) -> BasicError {
        Error::new("csv", format!("{:?}", err.kind()).as_str())
    }
}

impl From<serde_yaml::Error> for BasicError {
    fn from(err: serde_yaml::Error) -> Error<()> {
        Error::new("yaml", format!("{:?}", err).as_str())
    }
}
