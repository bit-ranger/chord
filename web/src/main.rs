use std::fs::File;
use std::path::Path;

use chord_common::error::Error;
use chord_common::rerr;
use chord_common::value::Json;

mod ctl;

mod app;
mod biz;

#[async_std::main]
async fn main() -> Result<(), Error> {
    let conf = load_conf("/data/chord/application.yml")?;
    app::init(conf).await?;
    Ok(())
}

pub fn load_conf<P: AsRef<Path>>(path: P) -> Result<Json, Error> {
    let file = File::open(path);
    let file = match file {
        Err(e) => return rerr!("yaml", format!("{:?}", e)),
        Ok(r) => r
    };

    let deserialized:Result<Json, serde_yaml::Error> = serde_yaml::from_reader(file);
    return match deserialized {
        Err(e) => return rerr!("yaml", format!("{:?}", e)),
        Ok(r) => Ok(r)
    };
}


