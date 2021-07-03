use log::debug;

use chord::action::prelude::*;
use chord::value::to_string_pretty;

pub struct LogFactory {}

impl LogFactory {
    pub async fn new(_: Option<Value>) -> Result<LogFactory, Error> {
        Ok(LogFactory {})
    }
}

#[async_trait]
impl Factory for LogFactory {
    async fn create(&self, _: &dyn CreateArg) -> Result<Box<dyn Action>, Error> {
        Ok(Box::new(Log {}))
    }
}

struct Log {}

#[async_trait]
impl Action for Log {
    async fn run(&self, arg: &dyn RunArg) -> Result<Box<dyn Scope>, Error> {
        let config = arg.render_str(to_string_pretty(arg.args())?.as_str())?;
        debug!("\n{}", config);
        return Ok(Box::new(Value::Null));
    }
}
