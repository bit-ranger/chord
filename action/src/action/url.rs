use chord_core::action::prelude::*;

use crate::err;

pub struct UrlFactory {}

impl UrlFactory {
    pub async fn new(_: Option<Value>) -> Result<UrlFactory, Error> {
        Ok(UrlFactory {})
    }
}

#[async_trait]
impl Factory for UrlFactory {
    async fn create(&self, _: &dyn CreateArg) -> Result<Box<dyn Action>, Error> {
        Ok(Box::new(Url {}))
    }
}

struct Url {}

#[async_trait]
impl Action for Url {
    async fn run(&self, arg: &dyn RunArg) -> Result<Box<dyn Scope>, Error> {
        let args = arg.args()?;
        let by = args["by"].as_str().ok_or(err!("100", "missing by"))?;

        let from = args["from"].as_str().ok_or(err!("101", "missing from"))?;

        return match by {
            "encode" => {
                let to = urlencoding::encode(from);
                Ok(Box::new(Value::String(to)))
            }
            "decode" => {
                let to = urlencoding::decode(from)?;
                Ok(Box::new(Value::String(to)))
            }
            _ => Err(err!("102", format!("unsupported {}", by))),
        };
    }
}
