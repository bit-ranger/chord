use chord_core::action::prelude::*;

use crate::err;

pub struct CryptoFactory {}

impl CryptoFactory {
    pub async fn new(_: Option<Value>) -> Result<CryptoFactory, Error> {
        Ok(CryptoFactory {})
    }
}

#[async_trait]
impl Factory for CryptoFactory {
    async fn create(&self, _: &dyn Arg) -> Result<Box<dyn Action>, Error> {
        Ok(Box::new(Crypto {}))
    }
}

struct Crypto {}

#[async_trait]
impl Action for Crypto {
    async fn run(&self, arg: &dyn Arg) -> Result<Box<dyn Scope>, Error> {
        run(arg).await
    }
}

async fn run(arg: &dyn Arg) -> Result<Box<dyn Scope>, Error> {
    let args = arg.args()?;
    let by = args["by"].as_str().ok_or(err!("100", "missing by"))?;

    let from = args["from"].as_str().ok_or(err!("101", "missing from"))?;

    return match by {
        "md5" => {
            let digest = md5::compute(from);
            let digest = format!("{:x}", digest);
            return Ok(Box::new(Value::String(digest)));
        }
        _ => Err(err!("102", format!("unsupported {}", by))),
    };
}
