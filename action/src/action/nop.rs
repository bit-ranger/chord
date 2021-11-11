use chord::action::prelude::*;

pub struct NopFactory {}

impl NopFactory {
    pub async fn new(_: Option<Value>) -> Result<NopFactory, Box<dyn Error>> {
        Ok(NopFactory {})
    }
}

#[async_trait]
impl Factory for NopFactory {
    async fn create(&self, _: &dyn CreateArg) -> Result<Box<dyn Action>, Box<dyn Error>> {
        Ok(Box::new(Nop {}))
    }
}

struct Nop {}

#[async_trait]
impl Action for Nop {
    async fn run(&self, _: &dyn RunArg) -> Result<Box<dyn Scope>, Box<dyn Error>> {
        return Ok(Box::new(Value::Null));
    }
}
