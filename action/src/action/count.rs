use std::sync::atomic::{AtomicU64, Ordering};

use crate::action::CommonScope;
use chord::action::prelude::*;

pub struct CountFactory {}

impl CountFactory {
    pub async fn new(_: Option<Value>) -> Result<CountFactory, Error> {
        Ok(CountFactory {})
    }
}

#[async_trait]
impl Factory for CountFactory {
    async fn create(&self, arg: &dyn CreateArg) -> Result<Box<dyn Action>, Error> {
        let init = arg.args_raw()["init"].as_u64().unwrap_or(1);
        let incr = arg.args_raw()["incr"].as_u64().unwrap_or(1);
        Ok(Box::new(Count {
            num: AtomicU64::new(init),
            incr,
        }))
    }
}

struct Count {
    num: AtomicU64,
    incr: u64,
}

#[async_trait]
impl Action for Count {
    async fn run(&self, _: &dyn RunArg) -> Result<Box<dyn Scope>, Error> {
        let value = Value::Number(Number::from(
            self.num.fetch_add(self.incr, Ordering::SeqCst),
        ));
        Ok(Box::new(CommonScope {
            args: Value::Null,
            value,
        }))
    }
}
