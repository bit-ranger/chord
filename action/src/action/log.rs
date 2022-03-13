use log::info;

use chord_core::action::prelude::*;

pub struct LogFactory {}

impl LogFactory {
    pub async fn new(_: Option<Value>) -> Result<LogFactory, Error> {
        Ok(LogFactory {})
    }
}

#[async_trait]
impl Factory for LogFactory {
    async fn create(&self, _: &dyn Arg) -> Result<Box<dyn Action>, Error> {
        Ok(Box::new(Log {}))
    }
}

struct Log {}

#[async_trait]
impl Action for Log {
    async fn run(&self, arg: &dyn Arg) -> Result<Box<dyn Scope>, Error> {
        let args = arg.args()?;
        info!("{}", args);
        return Ok(Box::new(Value::Null));
    }
}
