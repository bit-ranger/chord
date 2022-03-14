use chord_core::action::prelude::*;
use chord_core::action::Context;

use crate::err;

pub struct SetFactory {}

impl SetFactory {
    pub async fn new(_: Option<Value>) -> Result<SetFactory, Error> {
        Ok(SetFactory {})
    }
}

#[async_trait]
impl Factory for SetFactory {
    async fn create(&self, _: &dyn Arg) -> Result<Box<dyn Action>, Error> {
        Ok(Box::new(Set {}))
    }
}

struct Set {}

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
impl Action for Set {
    async fn run(&self, arg: &mut dyn Arg) -> Result<Box<dyn Scope>, Error> {
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
