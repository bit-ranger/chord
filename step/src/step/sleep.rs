use async_std::task::sleep;
use chord::rerr;
use chord::step::{async_trait, CreateArg, RunArg, StepRunner, StepRunnerFactory, StepValue};
use chord::value::Value;
use chord::Error;
use std::time::Duration;

pub struct Factory {}

impl Factory {
    pub async fn new(_: Option<Value>) -> Result<Factory, Error> {
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
        let sec = arg.render_value(&arg.config()["duration"])?;
        if sec.is_null() {
            return rerr!("sleep", "duration must > 0");
        }
        let sec = if sec.is_number() && sec.as_u64().is_some() {
            sec.as_u64().unwrap()
        } else if sec.is_string() {
            sec.as_str().unwrap().parse()?
        } else {
            0
        };

        if sec < 1 {
            return rerr!("sleep", "duration must > 0");
        }

        sleep(Duration::from_secs(sec)).await;
        return Ok(Value::Null);
    }
}
