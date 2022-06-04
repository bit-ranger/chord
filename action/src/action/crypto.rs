use chord_core::action::prelude::*;

use crate::err;

pub struct CryptoAction {}

impl CryptoAction {
    pub async fn new(_: Option<Value>) -> Result<CryptoAction, Error> {
        Ok(CryptoAction {})
    }
}

#[async_trait]
impl Action for CryptoAction {
    async fn play(&self, _: &dyn Arg) -> Result<Box<dyn Play>, Error> {
        Ok(Box::new(Crypto {}))
    }
}

struct Crypto {}

#[async_trait]
impl Play for Crypto {
    async fn execute(&self, arg: &mut dyn Arg) -> Result<Box<dyn Scope>, Error> {
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
