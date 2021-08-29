use std::fs::File;
use std::path::Path;

use chord::err;
use chord::input::FlowParse;
use chord::value::Value;
use chord::Error;
use log::{debug, trace};

pub fn load<P: AsRef<Path>>(path: P) -> Result<Value, Error> {
    let file = File::open(path);
    let file = match file {
        Err(e) => return Err(err!("yaml", format!("{:?}", e))),
        Ok(r) => r,
    };

    let deserialized: Result<Value, serde_yaml::Error> = serde_yaml::from_reader(file);
    return match deserialized {
        Err(e) => return Err(err!("yaml", format!("{:?}", e))),
        Ok(r) => Ok(r),
    };
}

pub fn from_str(txt: &str) -> Result<Value, Error> {
    let deserialized: Result<Value, serde_yaml::Error> = serde_yaml::from_str(txt);
    return match deserialized {
        Err(e) => return Err(err!("yaml", format!("{:?}", e))),
        Ok(r) => Ok(r),
    };
}

pub struct YmlFlowParser {}

impl YmlFlowParser {
    pub fn new() -> YmlFlowParser {
        YmlFlowParser {}
    }
}

impl FlowParse for YmlFlowParser {
    fn parse_str(&self, txt: &str) -> Result<Value, Error> {
        match from_str(txt) {
            Err(e) => {
                debug!("parse_str Err\n{}\n<<<\n{}", e, txt);
                Err(e)
            }
            Ok(r) => {
                trace!("parse_str Ok\n{}\n<<<\n{}", r, txt);
                Ok(r)
            }
        }
    }
}
