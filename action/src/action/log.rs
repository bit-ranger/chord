use log::info;

use chord_core::action::prelude::*;

pub struct LogAction {}

impl LogAction {
    pub async fn new(_: Option<Value>) -> Result<LogAction, Error> {
        Ok(LogAction {})
    }
}

#[async_trait]
impl Action for LogAction {
    async fn player(&self, _: &dyn Arg) -> Result<Box<dyn Player>, Error> {
        Ok(Box::new(Log {}))
    }
}

struct Log {}

#[async_trait]
impl Player for Log {
    async fn play(&self, arg: &mut dyn Arg) -> Result<Box<dyn Scope>, Error> {
        let args = arg.args()?;
        info!("{}", args);
        return Ok(Box::new(Value::Null));
    }
}
