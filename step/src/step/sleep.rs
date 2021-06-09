use async_std::task::sleep;
use chord_common::error::Error;
use chord_common::rerr;
use chord_common::step::{
    async_trait, CreateArg, RunArg, StepRunner, StepRunnerFactory, StepValue,
};
use chord_common::value::Json;
use std::time::Duration;

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
        let sec = arg.config()["duration"].as_u64();

        if sec.is_none() {
            return rerr!("sleep", "duration must > 0");
        }

        let sec = sec.unwrap();
        if sec < 1 {
            return rerr!("sleep", "duration must > 0");
        }

        sleep(Duration::from_secs(sec)).await;
        return Ok(Json::Null);
    }
}
