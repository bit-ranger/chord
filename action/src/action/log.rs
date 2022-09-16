use log::info;

use chord_core::action::prelude::*;

pub struct LogPlayer {}

impl LogPlayer {
    pub async fn new(_: Option<Value>) -> Result<LogPlayer, Error> {
        Ok(LogPlayer {})
    }
}

#[async_trait]
impl Player for LogPlayer {
    async fn action(&self, _: &dyn Arg) -> Result<Box<dyn Action>, Error> {
        Ok(Box::new(Log {}))
    }
}

struct Log {}

#[async_trait]
impl Action for Log {
    async fn run(&self, arg: &mut dyn Arg) -> Result<Box<dyn Scope>, Error> {
        let args = arg.body()?;
        info!("{}", args);
        return Ok(Box::new(Value::Null));
    }
}
