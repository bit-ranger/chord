use chord_core::action::prelude::*;

pub struct EchoFactory {}

impl EchoFactory {
    pub async fn new(_: Option<Value>) -> Result<EchoFactory, Error> {
        Ok(EchoFactory {})
    }
}

#[async_trait]
impl Factory for EchoFactory {
    async fn create(&self, _: &dyn CreateArg) -> Result<Box<dyn Action>, Error> {
        Ok(Box::new(Echo {}))
    }
}

struct Echo {}

#[async_trait]
impl Action for Echo {
    async fn run(&self, arg: &dyn RunArg) -> Result<Box<dyn Scope>, Error> {
        let args = arg.args()?;
        return Ok(Box::new(args.clone()));
    }
}
