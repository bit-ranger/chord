
use chord_core::action::prelude::*;

use crate::err;

pub struct CryptoCreator {}

impl CryptoCreator {
    pub async fn new(_: Option<Value>) -> Result<CryptoCreator, Error> {
        Ok(CryptoCreator {})
    }
}

#[async_trait]
impl Creator for CryptoCreator {
    async fn create(&self, _chord: &dyn Chord, _arg: &dyn Arg) -> Result<Box<dyn Action>, Error> {
        Ok(Box::new(Crypto {}))
    }
}

struct Crypto {}

#[async_trait]
impl Action for Crypto {
    async fn execute(
        &self,
        chord: &dyn Chord,
        arg: &mut dyn Arg,
    ) -> Result<Asset, Error> {
        run(arg).await
    }
}

async fn run(arg: &dyn Arg) -> Result<Asset, Error> {
    let args = arg.args()?;
    let by = args["by"].as_str().ok_or(err!("100", "missing by"))?;

    let from = args["from"].as_str().ok_or(err!("101", "missing from"))?;

    return match by {
        "md5" => {
            let digest = md5::compute(from);
            let digest = format!("{:x}", digest);
            return Ok(Asset::Value(Value::String(digest)));
        }
        _ => Err(err!("102", format!("unsupported {}", by))),
    };
}
