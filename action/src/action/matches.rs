use chord_core::action::prelude::*;
use chord_core::action::{Context, Id};
use log::trace;
use std::cell::RefCell;
use std::collections::HashMap;
use std::mem::replace;
use std::sync::Arc;
use std::time::Duration;

use crate::err;

pub struct MatchFactory {}

impl MatchFactory {
    pub async fn new(_: Option<Value>) -> Result<MatchFactory, Error> {
        Ok(MatchFactory {})
    }
}

struct Match {
    action: Box<dyn Action>,
}

struct MatchRunArg<'a> {
    delegate: &'a dyn Arg,
    index: usize,
    item: Value,
    context: Map,
}

impl<'a> MatchRunArg<'a> {
    fn new(delegate: &'a mut dyn Arg, index: usize, item: Value) -> MatchRunArg {
        let mut context = delegate.context().clone();
        context.insert("idx".to_string(), Value::Number(Number::from(index)));
        context.insert("item".to_string(), item.clone());
        MatchRunArg {
            delegate,
            index,
            item,
            context,
        }
    }
}

#[async_trait]
impl Factory for MatchFactory {
    async fn create(&self, arg: &dyn Arg) -> Result<Box<dyn Action>, Error> {
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
        let map_create_arg = MatchCreateArg {
            origin: arg,
            chosen: "".to_string(),
        };

        let factory = arg
            .factory(action)
            .ok_or(err!("102", format!("unsupported action {}", action)))?;

        let map_action = factory.create(&map_create_arg).await?;

        Ok(Box::new(Match { action: map_action }))
    }
}

#[async_trait]
impl Action for Match {
    async fn run(&self, arg: &dyn Arg) -> Result<Box<dyn Scope>, Error> {
        // let mut context = arg.context().clone();
        // context.insert("idx".to_string(), Value::Null);
        // context.insert("item".to_string(), Value::Null);

        let args = arg.args()?;
        trace!("{}", args);
        let array = args["iter"].as_array().ok_or(err!("103", "missing iter"))?;

        let mut map_val_vec = Vec::with_capacity(array.len());
        for (index, item) in array.iter().enumerate() {
            let mut mra = MatchRunArg::new(arg, index, item.clone());
            let val = self.action.run(&mut mra).await?;
            map_val_vec.push(val.as_value().clone());
        }
        Ok(Box::new(Value::Array(map_val_vec)))
    }
}
