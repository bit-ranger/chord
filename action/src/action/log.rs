use log::info;

use chord::action::prelude::*;

pub struct LogFactory {}

impl LogFactory {
    pub async fn new(_: Option<Value>) -> Result<LogFactory, Box<dyn Error>> {
        Ok(LogFactory {})
    }
}

#[async_trait]
impl Factory for LogFactory {
    async fn create(&self, _: &dyn CreateArg) -> Result<Box<dyn Action>, Box<dyn Error>> {
        Ok(Box::new(Log {}))
    }
}

struct Log {}

#[async_trait]
impl Action for Log {
    async fn run(&self, arg: &dyn RunArg) -> Result<Box<dyn Scope>, Box<dyn Error>> {
        let args = arg.args()?;
        info!("{}", args);
        return Ok(Box::new(Value::Null));
    }
}
