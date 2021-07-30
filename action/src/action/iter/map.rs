use async_std::sync::Arc;
use chord::action::prelude::*;
use chord::action::{CreateId, RunId};
use std::collections::HashMap;
use std::time::Duration;

pub struct IterMapFactory {
    table: HashMap<String, Arc<dyn Factory>>,
}

impl IterMapFactory {
    pub async fn new(
        _: Option<Value>,
        table: HashMap<String, Arc<dyn Factory>>,
    ) -> Result<IterMapFactory, Error> {
        Ok(IterMapFactory { table })
    }
}

struct MapperCreateArg<'a> {
    iter_arg: &'a dyn CreateArg,
}

impl<'a> CreateArg for MapperCreateArg<'a> {
    fn id(&self) -> &dyn CreateId {
        self.iter_arg.id()
    }

    fn action(&self) -> &str {
        self.iter_arg.action()
    }

    fn args(&self) -> &Value {
        &self.iter_arg.args()["map"]["args"]
    }

    fn render_str(&self, text: &str) -> Result<String, Error> {
        self.iter_arg.render_str(text)
    }

    fn is_shared(&self, text: &str) -> bool {
        self.iter_arg.is_shared(text)
    }
}

struct IterMap {
    item_action: Box<dyn Action>,
}

struct MapperRunArg<'a> {
    iter_arg: &'a dyn RunArg,
}

impl<'a> RunArg for MapperRunArg<'a> {
    fn id(&self) -> &dyn RunId {
        self.iter_arg.id()
    }

    fn args(&self) -> &Value {
        &self.iter_arg.args()["map"]["args"]
    }

    fn timeout(&self) -> Duration {
        self.iter_arg.timeout()
    }
}

#[async_trait]
impl Factory for IterMapFactory {
    async fn create(&self, arg: &dyn CreateArg) -> Result<Box<dyn Action>, Error> {
        let action = arg.args()["map"]["action"]
            .as_str()
            .ok_or(err!("101", "missing action"))?;
        let factory = self
            .table
            .get(action)
            .ok_or(err!("102", "unsupported action"))?;
        let item_create_arg = MapperCreateArg { iter_arg: arg };

        let item_action = factory.create(&item_create_arg).await?;
        Ok(Box::new(IterMap { item_action }))
    }
}

#[async_trait]
impl Action for IterMap {
    async fn run(&self, arg: &dyn RunArg) -> Result<Box<dyn Scope>, Error> {
        let iter = arg.args()["iter"]
            .as_array()
            .ok_or(err!("103", "missing iter"))?;
        let mut iter_val = Vec::with_capacity(iter.len());
        for item in iter {
            let mra = MapperRunArg { iter_arg: arg };
            let val = self.item_action.run(&mra).await?;
            iter_val.push(val.as_value().clone());
        }
        Ok(Box::new(Value::Array(iter_val)))
    }
}
