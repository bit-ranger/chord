use async_std::sync::Arc;
use chord::action::prelude::*;
use chord::action::{CreateId, RunId};
use itertools::Itertools;
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

struct MapCreateArg<'a> {
    iter_arg: &'a dyn CreateArg,
}

impl<'a> CreateArg for MapCreateArg<'a> {
    fn id(&self) -> &dyn CreateId {
        self.iter_arg.id()
    }

    fn action(&self) -> &str {
        self.iter_arg.action()
    }

    fn args_raw(&self) -> &Value {
        &self.iter_arg.args_raw()["map"]["args"]
    }

    fn render_str(&self, text: &str) -> Result<String, Error> {
        self.iter_arg.render_str(text)
    }

    fn is_shared(&self, text: &str) -> bool {
        self.iter_arg.is_shared(text)
    }
}

struct IterMap {
    action: Box<dyn Action>,
}

struct MapRunArg<'a, 'i> {
    iter_arg: &'a dyn RunArg,
    index: usize,
    index_name: String,
    item: &'i Value,
    item_name: String,
}

impl<'a, 'i> RunArg for MapRunArg<'a, 'i> {
    fn id(&self) -> &dyn RunId {
        self.iter_arg.id()
    }

    fn args(&self, ctx: Option<Map>) -> Result<Value, Error> {
        let mut ctx = ctx.unwrap_or(Map::new());
        ctx.insert(
            self.index_name.clone(),
            Value::Number(Number::from(self.index)),
        );
        ctx.insert(self.item_name.clone(), self.item.clone());
        Ok(self.iter_arg.args(Some(ctx))?["map"]["args"].clone())
    }

    fn timeout(&self) -> Duration {
        self.iter_arg.timeout()
    }
}

#[async_trait]
impl Factory for IterMapFactory {
    async fn create(&self, arg: &dyn CreateArg) -> Result<Box<dyn Action>, Error> {
        let action = arg.args_raw()["map"]["action"]
            .as_str()
            .ok_or(err!("101", "missing action"))?;
        let factory = self
            .table
            .get(action)
            .ok_or(err!("102", "unsupported action"))?;
        let map_create_arg = MapCreateArg { iter_arg: arg };

        let map_action = factory.create(&map_create_arg).await?;
        Ok(Box::new(IterMap { action: map_action }))
    }
}

#[async_trait]
impl Action for IterMap {
    async fn run(&self, arg: &dyn RunArg) -> Result<Box<dyn Scope>, Error> {
        let args = arg.args(None)?;
        let array = args["iter"]["arr"]
            .as_array()
            .ok_or(err!("103", "missing iter.arr"))?;
        let enumerate = args["iter"]["enum"]
            .as_str()
            .ok_or(err!("104", "missing iter.enum"))?;
        let (index_name, item_name) = enumerate
            .split(",")
            .collect_tuple()
            .ok_or(err!("105", "invalid iter.enum"))?;

        let mut iter_val = Vec::with_capacity(array.len());
        for (index, item) in array.iter().enumerate() {
            let mra = MapRunArg {
                iter_arg: arg,
                index,
                index_name: index_name.to_string(),
                item,
                item_name: item_name.to_string(),
            };
            let val = self.action.run(&mra).await?;
            iter_val.push(val.as_value().clone());
        }
        Ok(Box::new(Value::Array(iter_val)))
    }
}
