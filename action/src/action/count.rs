use std::sync::atomic::{AtomicU64, Ordering};

use chord_core::action::prelude::*;

pub struct CountAction {}

impl CountAction {
    pub async fn new(_: Option<Value>) -> Result<CountAction, Error> {
        Ok(CountAction {})
    }
}

#[async_trait]
impl Action for CountAction {
    async fn play(&self, arg: &dyn Arg) -> Result<Box<dyn Play>, Error> {
        let args_raw = arg.args_raw();
        let init = args_raw["init"].as_u64().unwrap_or(1);
        let incr = args_raw["incr"].as_u64().unwrap_or(1);
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
impl Play for Count {
    async fn execute(&self, _: &mut dyn Arg) -> Result<Box<dyn Scope>, Error> {
        Ok(Box::new(Value::Number(Number::from(
            self.num.fetch_add(self.incr, Ordering::SeqCst),
        ))))
    }
}
