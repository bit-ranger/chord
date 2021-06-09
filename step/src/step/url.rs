use chord_common::error::Error;
use chord_common::step::{
    async_trait, CreateArg, RunArg, StepRunner, StepRunnerFactory, StepValue,
};
use chord_common::value::Json;
use chord_common::{err, rerr};

pub struct Factory {}

impl Factory {
    pub async fn new(_: Option<Json>) -> Result<Factory, Error> {
        Ok(Factory {})
    }
}

#[async_trait]
impl StepRunnerFactory for Factory {
    async fn create(&self, _: &dyn CreateArg) -> Result<Box<dyn StepRunner>, Error> {
        Ok(Box::new(Runner {}))
    }
}

struct Runner {}

#[async_trait]
impl StepRunner for Runner {
    async fn run(&self, arg: &dyn RunArg) -> StepValue {
        let from = arg.config()["from"]
            .as_str()
            .map(|s| arg.render(s))
            .ok_or(err!("010", "missing from"))??;

        let by = arg.config()["by"]
            .as_str()
            .ok_or(err!("010", "missing by"))?;

        return match by {
            "encode" => {
                let to = urlencoding::encode(from.as_str());
                Ok(Json::String(to))
            }
            "decode" => {
                let to = urlencoding::decode(from.as_str())?;
                Ok(Json::String(to))
            }
            _ => {
                rerr!("url", format!("unsupported {}", by))
            }
        };
    }
}
