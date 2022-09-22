use chord_core::action::prelude::*;
use chord_core::action::Context;

use crate::err;

pub struct AlterCreator {}

impl AlterCreator {
    pub async fn new(_: Option<Value>) -> Result<AlterCreator, Error> {
        Ok(AlterCreator {})
    }
}

#[async_trait]
impl Creator for AlterCreator {
    async fn create(&self, _chord: &dyn Chord, _arg: &dyn Arg) -> Result<Box<dyn Action>, Error> {
        Ok(Box::new(Alter {}))
    }
}

struct Alter {}

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
impl Action for Alter {
    async fn execute(
        &self,
        _chord: &dyn Chord,
        arg: &mut dyn Arg,
    ) -> Result<Box<dyn Scope>, Error> {
        let args = arg.args()?;
        let obj = args
            .as_object()
            .ok_or(err!("100", "alter must be a object"))?;
        for (k, v) in obj {
            arg.context_mut()
                .data_mut()
                .insert(k.to_string(), v.clone());
        }

        Ok(Box::new(Value::Null))
    }
}
