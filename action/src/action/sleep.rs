use std::time::Duration;

use chord_core::action::prelude::*;
use chord_core::future::time::sleep;

use crate::err;

pub struct SleepCreator {}

impl SleepCreator {
    pub async fn new(_: Option<Value>) -> Result<SleepCreator, Error> {
        Ok(SleepCreator {})
    }
}

#[async_trait]
impl Creator for SleepCreator {
    async fn create(&self, _: &dyn Arg) -> Result<Box<dyn Action>, Error> {
        Ok(Box::new(Sleep {}))
    }
}

struct Sleep {}

#[async_trait]
impl Action for Sleep {
    async fn execute(&self, arg: &mut dyn Arg) -> Result<Box<dyn Scope>, Error> {
        let sec = arg.body()?;
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
