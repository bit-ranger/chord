use std::fs::File;
use std::path::Path;
use std::path::PathBuf;

use structopt::StructOpt;

use chord_common::error::Error;
use chord_common::rerr;
use chord_common::value::Json;

mod ctl;

mod app;
mod biz;

#[async_std::main]
async fn main() -> Result<(), Error> {
    let opt = Opt::from_args();
    let conf = load_conf(&opt.config)?;
    app::init(conf).await?;
    Ok(())
}

pub fn load_conf<P: AsRef<Path>>(path: P) -> Result<Json, Error> {
    let file = File::open(path);
    let file = match file {
        Err(_) => return Ok(Json::Null),
        Ok(r) => r,
    };

    let deserialized: Result<Json, serde_yaml::Error> = serde_yaml::from_reader(file);
    return match deserialized {
        Err(e) => return rerr!("yaml", format!("{:?}", e)),
        Ok(r) => Ok(r),
    };
}

#[derive(StructOpt, Debug)]
#[structopt(name = "chord")]
struct Opt {
    /// config file path
    #[structopt(
        short,
        long,
        parse(from_os_str),
        default_value = "/data/chord/conf/application.yml"
    )]
    config: PathBuf,
}
