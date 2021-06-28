use chord::action::prelude::*;
use log::debug;

pub struct LogFactory {}

impl LogFactory {
    pub async fn new(_: Option<Value>) -> Result<LogFactory, Error> {
        Ok(LogFactory {})
    }
}

#[async_trait]
impl Factory for LogFactory {
    async fn create(&self, _: &dyn CreateArg) -> Result<Box<dyn Action>, Error> {
        Ok(Box::new(Echo {}))
    }
}

struct Echo {}

#[async_trait]
impl Action for Echo {
    async fn run(&self, arg: &dyn RunArg) -> ActionValue {
        let config = arg.render_value(arg.args())?;
        debug!("{}", config);
        return Ok(Value::Null);
    }
}
