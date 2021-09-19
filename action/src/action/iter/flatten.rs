use crate::action::CommonScope;
use chord::action::prelude::*;
use log::trace;

pub struct IterFlattenFactory {}

impl IterFlattenFactory {
    pub async fn new(_: Option<Value>) -> Result<IterFlattenFactory, Error> {
        Ok(IterFlattenFactory {})
    }
}

#[async_trait]
impl Factory for IterFlattenFactory {
    async fn create(&self, _: &dyn CreateArg) -> Result<Box<dyn Action>, Error> {
        Ok(Box::new(IterFlatten {}))
    }
}

struct IterFlatten {}

#[async_trait]
impl Action for IterFlatten {
    async fn run(&self, arg: &dyn RunArg) -> Result<Box<dyn Scope>, Error> {
        let args = arg.args(None)?;
        trace!("{}", args);
        let array = args["arr"].as_array().ok_or(err!("103", "missing .arr"))?;

        let mut vec_vec = Vec::with_capacity(array.len());
        for arr in array {
            let vec = arr.as_array().ok_or(err!("103", "invalid item in arr"))?;
            vec_vec.push(vec.clone());
        }

        let flat_val_vec = vec_vec.into_iter().flatten().collect();
        let value = Value::Array(flat_val_vec);
        Ok(Box::new(CommonScope { args, value }))
    }
}
