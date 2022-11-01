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
    async fn create(&self, _chord: &dyn Chord, _arg: &dyn Arg) -> Result<Box<dyn Action>, Error> {
        Ok(Box::new(Log {}))
    }
}

struct Log {}

#[async_trait]
impl Action for Log {
    async fn execute(
        &self,
        chord: &dyn Chord,
        arg: &mut dyn Arg,
    ) -> Result<Asset, Error> {
        let args = arg.args()?;
        info!("{}", args);
        return Ok(Asset::Value(Value::Null));
    }
}
