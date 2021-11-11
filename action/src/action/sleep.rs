use std::time::Duration;

use async_std::task::sleep;

use chord::action::prelude::*;

pub struct SleepFactory {}

impl SleepFactory {
    pub async fn new(_: Option<Value>) -> Result<SleepFactory, Box<dyn Error>> {
        Ok(SleepFactory {})
    }
}

#[async_trait]
impl Factory for SleepFactory {
    async fn create(&self, _: &dyn CreateArg) -> Result<Box<dyn Action>, Box<dyn Error>> {
        Ok(Box::new(Sleep {}))
    }
}

struct Sleep {}

#[async_trait]
impl Action for Sleep {
    async fn run(&self, arg: &dyn RunArg) -> Result<Box<dyn Scope>, Box<dyn Error>> {
        let sec = arg.args()?;
        if sec.is_null() {
            return Err(err!("100", "sleep must > 0"));
        }
        let sec = if sec.is_number() && sec.as_u64().is_some() {
            sec.as_u64().unwrap()
        } else if sec.is_string() {
            sec.as_str().unwrap().parse()?
        } else {
            0
        };

        if sec < 1 {
            return Err(err!("101", "sleep must > 0"));
        }

        sleep(Duration::from_secs(sec)).await;
        return Ok(Box::new(Value::Null));
    }
}
