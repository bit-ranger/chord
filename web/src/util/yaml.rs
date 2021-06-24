use async_std::fs::File;
use async_std::path::Path;
use chord::rerr;
use chord::value::Value;
use chord::Error;
use futures::AsyncReadExt;

pub async fn load<P: AsRef<Path>>(path: P) -> Result<Value, Error> {
    let file = File::open(path).await;
    let mut file = match file {
        Err(_) => return Ok(Value::Null),
        Ok(r) => r,
    };
    let mut content = String::new();
    file.read_to_string(&mut content).await?;

    let deserialized: Result<Value, serde_yaml::Error> = serde_yaml::from_str(content.as_str());
    return match deserialized {
        Err(e) => return rerr!("yaml", format!("{:?}", e)),
        Ok(r) => Ok(r),
    };
}
