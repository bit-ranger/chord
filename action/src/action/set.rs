use chord_core::action::prelude::*;
use chord_core::action::Context;

use crate::err;

pub struct SetAction {}

impl SetAction {
    pub async fn new(_: Option<Value>) -> Result<SetAction, Error> {
        Ok(SetAction {})
    }
}

#[async_trait]
impl Action for SetAction {
    async fn play(&self, _: &dyn Arg) -> Result<Box<dyn Play>, Error> {
        Ok(Box::new(Set {}))
    }
}

struct Set {}

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
impl Play for Set {
    async fn execute(&self, arg: &mut dyn Arg) -> Result<Box<dyn Scope>, Error> {
        let args = arg.args()?;
        let obj = args
            .as_object()
            .ok_or(err!("100", "set must be a object"))?;
        for (k, v) in obj {
            arg.context_mut()
                .data_mut()
                .insert(k.to_string(), v.clone());
        }

        Ok(Box::new(Value::Null))
    }
}
