use std::fs::File;
use std::path::Path;

use common::error::Error;
use common::value::Json;
use common::err;

pub fn load<P: AsRef<Path>>(path: P) -> Result<Json, Error> {
    let file = File::open(path);
    let file = match file {
        Err(e) => return err!("yaml", format!("{:?}", e)),
        Ok(r) => r
    };

    let deserialized:Result<Json, serde_yaml::Error> = serde_yaml::from_reader(file);
    return match deserialized {
        Err(e) => return err!("yaml", format!("{:?}", e)),
        Ok(r) => Ok(r)
    };
}
