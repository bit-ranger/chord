use async_std::path::PathBuf;

use structopt::StructOpt;

use crate::util::yaml::load;
use chord::Error;

mod ctl;

mod app;
mod util;

#[async_std::main]
async fn main() -> Result<(), Error> {
    let opt = Opt::from_args();
    let conf_path = opt
        .config
        .clone()
        .map(|p| PathBuf::from(p))
        .unwrap_or_else(|| {
            PathBuf::from(
                dirs::home_dir()
                    .unwrap()
                    .join(".chord")
                    .join("conf")
                    .join("web.yml"),
            )
        });
    let conf = load(conf_path).await?;
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
