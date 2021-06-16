use chord::rerr;
use chord::value::Value;
use chord::Error;
use std::fs::File;
use std::path::Path;

pub async fn load<P: AsRef<Path>>(path: P) -> Result<Value, Error> {
    let file = File::open(path);
    let file = match file {
        Err(_) => return Ok(Value::Null),
        Ok(r) => r,
    };

    let deserialized: Result<Value, serde_yaml::Error> = serde_yaml::from_reader(file);
    return match deserialized {
        Err(e) => return rerr!("yaml", format!("{:?}", e)),
        Ok(r) => Ok(r),
    };
}
