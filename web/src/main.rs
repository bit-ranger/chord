use std::path::PathBuf;

use structopt::StructOpt;

use app::Error;

mod ctl;

mod app;

#[chord_core::future::main]
async fn main() -> Result<(), Error> {
    let opt = Opt::from_args();
    let conf_dir_path = opt
        .config
        .clone()
        .map(|p| PathBuf::from(p))
        .unwrap_or_else(|| PathBuf::from(dirs::home_dir().unwrap().join(".chord").join("conf")));
    let conf = chord_input::conf::load(conf_dir_path, "web")
        .await
        .map_err(|e| Error::Config(e))?;
    app::init(conf).await?;
    Ok(())
}

#[derive(StructOpt, Debug)]
#[structopt(name = "chord")]
struct Opt {
    /// config file path
    #[structopt(short, long, parse(from_os_str))]
    config: Option<PathBuf>,
}
