use chord_core::action::prelude::*;

use crate::err;

pub struct UrlAction {}

impl UrlAction {
    pub async fn new(_: Option<Value>) -> Result<UrlAction, Error> {
        Ok(UrlAction {})
    }
}

#[async_trait]
impl Action for UrlAction {
    async fn player(&self, _: &dyn Arg) -> Result<Box<dyn Player>, Error> {
        Ok(Box::new(Url {}))
    }
}

struct Url {}

#[async_trait]
impl Player for Url {
    async fn play(&self, arg: &mut dyn Arg) -> Result<Box<dyn Scope>, Error> {
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
