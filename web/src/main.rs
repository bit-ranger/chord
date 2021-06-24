use async_std::path::PathBuf;

use structopt::StructOpt;

use crate::util::yaml::load;
use chord::Error;

mod ctl;

mod app;
mod biz;
mod util;

#[async_std::main]
async fn main() -> Result<(), Error> {
    let opt = Opt::from_args();
    let conf = load(&opt.config).await?;
    app::init(conf).await?;
    Ok(())
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
