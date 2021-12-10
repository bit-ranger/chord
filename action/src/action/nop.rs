use chord_core::action::prelude::*;

pub struct NopFactory {}

impl NopFactory {
    pub async fn new(_: Option<Value>) -> Result<NopFactory, Error> {
        Ok(NopFactory {})
    }
}

#[async_trait]
impl Factory for NopFactory {
    async fn create(&self, _: &dyn CreateArg) -> Result<Box<dyn Action>, Error> {
        Ok(Box::new(Nop {}))
    }
}

struct Nop {}

#[async_trait]
impl Action for Nop {
    async fn run(&self, _: &dyn RunArg) -> Result<Box<dyn Scope>, Error> {
        return Ok(Box::new(Value::Null));
    }
}
