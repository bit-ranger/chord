
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
    async fn create(&self, _chord: &dyn Chord, _arg: &dyn Arg) -> Result<Box<dyn Action>, Error> {
        Ok(Box::new(Url {}))
    }
}

struct Url {}

#[async_trait]
impl Action for Url {
    async fn execute(
        &self,
        chord: &dyn Chord,
        arg: &mut dyn Arg,
    ) -> Result<Asset, Error> {
        let args = arg.args()?;
        let by = args["by"].as_str().ok_or(err!("100", "missing by"))?;

        let from = args["from"].as_str().ok_or(err!("101", "missing from"))?;

        return match by {
            "encode" => {
                let to = urlencoding::encode(from);
                Ok(Asset::Value(Value::String(to)))
            }
            "decode" => {
                let to = urlencoding::decode(from)?;
                Ok(Asset::Value(Value::String(to)))
            }
            _ => Err(err!("102", format!("unsupported {}", by))),
        };
    }
}
