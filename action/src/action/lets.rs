use chord_core::action::prelude::*;
use chord_core::action::Context;

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

struct ContextStruct {
    map: Map,
}

impl Context for ContextStruct {
    fn data(&self) -> &Map {
        &self.map
    }

    fn data_mut(&mut self) -> &mut Map {
        &mut self.map
    }
}

#[async_trait]
impl Action for Let {
    async fn run(&self, arg: &mut dyn RunArg) -> Result<Box<dyn Scope>, Error> {
        let mut lets = Map::new();
        if arg.args_raw().is_object() {
            let mut new_ctx = ContextStruct {
                map: arg.context().data().clone(),
            };
            for (k, v) in arg.args_raw().as_object().unwrap() {
                let rvr = arg.render(&new_ctx, v)?;
                new_ctx.data_mut().insert(k.clone(), rvr.clone());
                lets.insert(k.clone(), rvr);
            }
            Ok(Box::new(Value::Object(lets)))
        } else {
            Ok(Box::new(arg.args()?))
        }
    }
}
