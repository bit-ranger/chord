use chord_core::action::prelude::*;

use crate::err;

pub struct UrlCreator {}

impl UrlCreator {
    pub async fn new(_: Option<Value>) -> Result<UrlCreator, Error> {
        Ok(UrlCreator {})
    }
}

#[async_trait]
impl Creator for UrlCreator {
    async fn create(&self, _: &dyn Arg) -> Result<Box<dyn Action>, Error> {
        Ok(Box::new(Url {}))
    }
}

struct Url {}

#[async_trait]
impl Action for Url {
    async fn execute(&self, arg: &mut dyn Arg) -> Result<Box<dyn Scope>, Error> {
        let args = arg.body()?;
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
