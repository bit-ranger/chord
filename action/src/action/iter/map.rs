use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use log::trace;

use chord_core::action::{CreateId, RunId};
use chord_core::action::prelude::*;

use crate::err;

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
    args_raw: Value,
}

impl<'a> CreateArg for MapCreateArg<'a> {
    fn id(&self) -> &dyn CreateId {
        self.iter_arg.id()
    }

    fn action(&self) -> &str {
        self.iter_arg.action()
    }

    fn args_raw(&self) -> &Value {
        &self.args_raw["map"]["args"]
    }

    fn render_str(&self, text: &str) -> Result<Value, Error> {
        self.iter_arg.render_str(text)
    }

    fn is_static(&self, text: &str) -> bool {
        self.iter_arg.is_static(text)
    }
}

struct IterMap {
    map_action: Box<dyn Action>,
}

struct MapRunArg<'a> {
    delegate: &'a dyn RunArg,
    index: usize,
    item: Value,
    context: Map,
}

impl<'a> MapRunArg<'a> {
    fn new(delegate: &'a mut dyn RunArg, index: usize, item: Value) -> MapRunArg {
        let mut context = delegate.context().clone();
        context.insert("idx".to_string(), Value::Number(Number::from(index)));
        context.insert("item".to_string(), item.clone());
        MapRunArg {
            delegate,
            index,
            item,
            context,
        }
    }
}

impl<'a> RunArg for MapRunArg<'a> {
    fn id(&self) -> &dyn RunId {
        self.delegate.id()
    }


    fn context(&mut self) -> &mut Map {
        &mut self.context
    }

    fn args(&self) -> Result<Value, Error> {
        self.delegate.args()
    }
}

#[async_trait]
impl Factory for IterMapFactory {
    async fn create(&self, arg: &dyn CreateArg) -> Result<Box<dyn Action>, Error> {
        let args_raw = arg.args_raw();
        let map = args_raw["map"]
            .as_object()
            .ok_or(err!("101", "missing map"))?;
        if map.is_empty() {
            return Err(err!("102", "missing iter_map.map"));
        }

        if map.len() != 1 {
            return Err(err!("102", "invalid iter_map.map"));
        }

        let action = map.keys().nth(0).unwrap().as_str();
        let factory = match action {
            "iter_map" => self as &dyn Factory,
            _ => self
                .table
                .get(action)
                .ok_or(err!("102", format!("unsupported action {}", action)))?
                .as_ref(),
        };

        let map_create_arg = MapCreateArg {
            iter_arg: arg,
            args_raw: arg.args_raw().clone(),
        };

        let map_action = factory.create(&map_create_arg).await?;

        Ok(Box::new(IterMap { map_action }))
    }
}

#[async_trait]
impl Action for IterMap {
    async fn run(&self, arg: &mut dyn RunArg) -> Result<Box<dyn Scope>, Error> {
        // let mut context = arg.context().clone();
        // context.insert("idx".to_string(), Value::Null);
        // context.insert("item".to_string(), Value::Null);

        let args = arg.args()?;
        trace!("{}", args);
        let array = args["iter"].as_array().ok_or(err!("103", "missing iter"))?;

        let mut map_val_vec = Vec::with_capacity(array.len());
        for (index, item) in array.iter().enumerate() {
            let mut mra = MapRunArg::new(arg, index, item.clone());
            let val = self.map_action.run(&mut mra).await?;
            map_val_vec.push(val.as_value().clone());
        }
        Ok(Box::new(Value::Array(map_val_vec)))
    }
}
