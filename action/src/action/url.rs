use chord::action::prelude::*;
use chord::{err, rerr};

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
    async fn run(&self, arg: &dyn RunArg) -> ActionValue {
        let by = arg.args()["by"].as_str().ok_or(err!("010", "missing by"))?;

        let from = arg.args()["from"]
            .as_str()
            .map(|s| arg.render_str(s))
            .ok_or(err!("010", "missing from"))??;

        return match by {
            "encode" => {
                let to = urlencoding::encode(from.as_str());
                Ok(Value::String(to))
            }
            "decode" => {
                let to = urlencoding::decode(from.as_str())?;
                Ok(Value::String(to))
            }
            _ => {
                rerr!("url", format!("unsupported {}", by))
            }
        };
    }
}
