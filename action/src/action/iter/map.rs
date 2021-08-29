use async_std::sync::Arc;
use chord::action::prelude::*;
use chord::action::{Context, CreateId, RunId};
use itertools::Itertools;
use log::trace;
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

    fn render_str(&self, text: &str, ctx: Option<Box<dyn Context>>) -> Result<String, Error> {
        self.iter_arg.render_str(text, ctx)
    }

    fn is_shared(&self, text: &str) -> bool {
        self.iter_arg.is_shared(text)
    }
}

struct IterMap {
    map_action: Box<dyn Action>,
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

    fn args(&self, ctx: Option<Box<dyn Context>>) -> Result<Value, Error> {
        let map_ctx = MapContext {
            upper_ctx: ctx,
            index: self.index,
            index_name: self.index_name.clone(),
            item: self.item.clone(),
            item_name: self.item_name.clone(),
        };
        Ok(self.iter_arg.args(Some(Box::new(map_ctx)))?["map"]["args"].clone())
    }

    fn timeout(&self) -> Duration {
        self.iter_arg.timeout()
    }

    fn render_str(&self, text: &str, ctx: Option<Box<dyn Context>>) -> Result<String, Error> {
        self.iter_arg.render_str(text, ctx)
    }
}

struct MapContext {
    upper_ctx: Option<Box<dyn Context>>,
    index: usize,
    index_name: String,
    item: Value,
    item_name: String,
}

impl Context for MapContext {
    fn update(&self, value: &mut Value) {
        value[self.index_name.as_str()] = Value::Number(Number::from(self.index));
        value[self.item_name.as_str()] = self.item.clone();
        if let Some(ctx) = self.upper_ctx.as_ref() {
            ctx.update(value);
        }
    }
}

#[async_trait]
impl Factory for IterMapFactory {
    async fn create(&self, arg: &dyn CreateArg) -> Result<Box<dyn Action>, Error> {
        let map = arg.args_raw()["map"]
            .as_object()
            .ok_or(err!("101", "missing map"))?;

        let action = map["action"]
            .as_str()
            .ok_or(err!("102", "missing action"))?;
        let factory = match action {
            "iter_map" => self as &dyn Factory,
            _ => self
                .table
                .get(action)
                .ok_or(err!("102", format!("unsupported action {}", action)))?
                .as_ref(),
        };

        let map_create_arg = MapCreateArg { iter_arg: arg };

        let map_action = factory.create(&map_create_arg).await?;

        Ok(Box::new(IterMap { map_action }))
    }
}

#[async_trait]
impl Action for IterMap {
    async fn run(&self, arg: &dyn RunArg) -> Result<Box<dyn Scope>, Error> {
        let args = arg.args(None)?;
        trace!("{}", args);
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

        let mut map_val_vec = Vec::with_capacity(array.len());
        for (index, item) in array.iter().enumerate() {
            let mra = MapRunArg {
                iter_arg: arg,
                index,
                index_name: index_name.to_string(),
                item,
                item_name: item_name.to_string(),
            };
            let val = self.map_action.run(&mra).await?;
            map_val_vec.push(val.as_value().clone());
        }
        Ok(Box::new(Value::Array(map_val_vec)))
    }
}
