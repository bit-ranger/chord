use chord::action::{Action, ActionFactory, ActionValue, CreateArg, RunArg};
use chord::step::async_trait;
use chord::value::Value;
use chord::Error;
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
        Ok(Box::new(Url {}))
    }
}

struct Url {}

#[async_trait]
impl Action for Url {
    async fn run(&self, arg: &dyn RunArg) -> ActionValue {
        let by = arg.config()["by"]
            .as_str()
            .ok_or(err!("010", "missing by"))?;

        let from = arg.config()["from"]
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
