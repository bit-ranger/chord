use async_std::path::PathBuf;

use structopt::StructOpt;

use chord::Error;
use chord_input::load::conf;

mod ctl;

mod app;

#[async_std::main]
async fn main() -> Result<(), Error> {
    let opt = Opt::from_args();
    let conf_dir_path = opt
        .config
        .clone()
        .map(|p| PathBuf::from(p))
        .unwrap_or_else(|| PathBuf::from(dirs::home_dir().unwrap().join(".chord").join("conf")));
    let conf = conf::load(conf_dir_path, "web").await?;
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
