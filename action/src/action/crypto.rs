use crate::action::CommonScope;
use chord::action::prelude::*;

pub struct CryptoFactory {}

impl CryptoFactory {
    pub async fn new(_: Option<Value>) -> Result<CryptoFactory, Error> {
        Ok(CryptoFactory {})
    }
}

#[async_trait]
impl Factory for CryptoFactory {
    async fn create(&self, _: &dyn CreateArg) -> Result<Box<dyn Action>, Error> {
        Ok(Box::new(Crypto {}))
    }
}

struct Crypto {}

#[async_trait]
impl Action for Crypto {
    async fn run(&self, arg: &dyn RunArg) -> Result<Box<dyn Scope>, Error> {
        run(arg).await
    }
}

async fn run(arg: &dyn RunArg) -> Result<Box<dyn Scope>, Error> {
    let args = arg.args(None)?;
    let by = args["by"].as_str().ok_or(err!("100", "missing by"))?;

    let from = args["from"].as_str().ok_or(err!("101", "missing from"))?;

    return match by {
        "md5" => {
            let digest = md5::compute(from);
            let digest = format!("{:x}", digest);
            let value = Value::String(digest);
            return Ok(Box::new(CommonScope { args, value }));
        }
        _ => Err(err!("102", format!("unsupported {}", by))),
    };
}
