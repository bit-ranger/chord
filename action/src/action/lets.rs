use chord_core::action::prelude::*;

pub struct LetFactory {}

impl LetFactory {
    pub async fn new(_: Option<Value>) -> Result<LetFactory, Error> {
        Ok(LetFactory {})
    }
}

#[async_trait]
impl Factory for LetFactory {
    async fn create(&self, _: &dyn CreateArg) -> Result<Box<dyn Action>, Error> {
        Ok(Box::new(Let {}))
    }
}

struct Let {}

#[async_trait]
impl Action for Let {
    async fn run(&self, arg: &mut dyn RunArg) -> Result<Box<dyn Scope>, Error> {
        Ok(Box::new(arg.args()?))
    }
}
