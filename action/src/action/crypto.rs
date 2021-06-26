use chord::action::prelude::*;

use chord::{err, rerr};

pub struct Factory {}

impl Factory {
    pub async fn new(_: Option<Value>) -> Result<Factory, Error> {
        Ok(Factory {})
    }
}

#[async_trait]
impl ActionFactory for Factory {
    async fn create(&self, _: &dyn CreateArg) -> Result<Box<dyn Action>, Error> {
        Ok(Box::new(Crypto {}))
    }
}

struct Crypto {}

#[async_trait]
impl Action for Crypto {
    async fn run(&self, arg: &dyn RunArg) -> ActionValue {
        run(arg).await
    }
}

async fn run(arg: &dyn RunArg) -> ActionValue {
    let by = arg.config()["by"]
        .as_str()
        .ok_or(err!("010", "missing by"))?;

    let from = arg.config()["from"]
        .as_str()
        .map(|s| arg.render_str(s))
        .ok_or(err!("010", "missing from"))??;

    return match by {
        "md5" => {
            let digest = md5::compute(from);
            let digest = format!("{:x}", digest);
            return Ok(Value::String(digest));
        }
        _ => {
            rerr!("crypto", format!("unsupported {}", by))
        }
    };
}
