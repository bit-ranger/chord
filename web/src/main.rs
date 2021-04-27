use std::fs::File;
use std::path::Path;

use chord_common::error::Error;
use chord_common::rerr;
use chord_common::value::Json;
use std::env;

mod ctl;

mod app;
mod biz;

#[async_std::main]
async fn main() -> Result<(), Error> {
    let args: Vec<_> = env::args().collect();
    let mut opts = getopts::Options::new();
    opts.optopt("c", "conf", "config path", "conf");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(e) => {
            println!("{}", opts.short_usage("chord"));
            return rerr!("arg", e.to_string());
        }
    };

    let conf_path = matches.opt_get_default("c", "/data/chord/conf/application.yml".to_owned()).unwrap();
    let conf = load_conf(conf_path)?;
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


