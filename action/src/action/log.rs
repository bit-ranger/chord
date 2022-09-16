use log::info;

use chord_core::action::prelude::*;

pub struct LogCreator {}

impl LogCreator {
    pub async fn new(_: Option<Value>) -> Result<LogCreator, Error> {
        Ok(LogCreator {})
    }
}

#[async_trait]
impl Creator for LogCreator {
    async fn create(&self, _: &dyn Arg) -> Result<Box<dyn Action>, Error> {
        Ok(Box::new(Log {}))
    }
}

struct Log {}

#[async_trait]
impl Action for Log {
    async fn execute(&self, arg: &mut dyn Arg) -> Result<Box<dyn Scope>, Error> {
        let args = arg.body()?;
        info!("{}", args);
        return Ok(Box::new(Value::Null));
    }
}
