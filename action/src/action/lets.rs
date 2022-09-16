use chord_core::action::prelude::*;
use chord_core::action::Context;

pub struct LetCreator {}

impl LetCreator {
    pub async fn new(_: Option<Value>) -> Result<LetCreator, Error> {
        Ok(LetCreator {})
    }
}

#[async_trait]
impl Creator for LetCreator {
    async fn create(&self, _: &dyn Arg) -> Result<Box<dyn Action>, Error> {
        Ok(Box::new(Let {}))
    }
}

struct Let {}

#[derive(Clone)]
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

    fn clone(&self) -> Box<dyn Context> {
        let ctx = Clone::clone(self);
        Box::new(ctx)
    }
}

#[async_trait]
impl Action for Let {
    async fn execute(&self, arg: &mut dyn Arg) -> Result<Box<dyn Scope>, Error> {
        let mut lets = Map::new();
        if arg.body_raw().is_object() {
            let mut new_ctx = ContextStruct {
                map: arg.context().data().clone(),
            };
            for (k, v) in arg.body_raw().as_object().unwrap() {
                let rvr = arg.render(&new_ctx, v)?;
                new_ctx.data_mut().insert(k.clone(), rvr.clone());
                lets.insert(k.clone(), rvr);
            }
            Ok(Box::new(Value::Object(lets)))
        } else {
            Ok(Box::new(arg.body()?))
        }
    }
}
