use crate::err;
use async_std::sync::Arc;
use chord::action::prelude::*;
use chord::action::{CreateId, RunId};
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
    ) -> Result<IterMapFactory, Box<dyn Error>> {
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
        &self.args_raw["exec"]["args"]
    }

    fn render_str(&self, text: &str) -> Result<Value, Box<dyn Error>> {
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
    fn new(delegate: &'a dyn RunArg, index: usize, item: Value) -> MapRunArg {
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

    fn timeout(&self) -> Duration {
        self.delegate.timeout()
    }

    fn context(&self) -> &Map {
        &self.context
    }

    fn args(&self) -> Result<Value, Box<dyn Error>> {
        self.args_with(self.context())
    }

    fn args_with(&self, context: &Map) -> Result<Value, Box<dyn Error>> {
        let mut ctx = context.clone();
        ctx.insert("idx".to_string(), Value::Number(Number::from(self.index)));
        ctx.insert("item".to_string(), self.item.clone());
        let args = self.delegate.args_with(&ctx)?;
        if let Some(map) = args.get("exec") {
            if let Value::Object(exec) = map {
                let step_id = self.id().step();
                if exec.is_empty() {
                    return Err(err!(
                        "flow",
                        format!("step {} missing exec.action", step_id)
                    ));
                }

                if exec.len() != 1 {
                    return Err(err!(
                        "flow",
                        format!("step {} invalid exec.action", step_id)
                    ));
                }

                return Ok(exec.values().nth(0).unwrap().clone());
            }
        }
        return Ok(Value::Null);
    }
}

#[async_trait]
impl Factory for IterMapFactory {
    async fn create(&self, arg: &dyn CreateArg) -> Result<Box<dyn Action>, Box<dyn Error>> {
        let args_raw = arg.args_raw();
        let exec = args_raw["exec"]
            .as_object()
            .ok_or(err!("101", "missing exec"))?;
        if exec.is_empty() {
            return Err(err!("102", "missing iter_map.exec.action"));
        }

        if exec.len() != 1 {
            return Err(err!("102", "invalid iter_map.exec.action"));
        }

        let action = exec.keys().nth(0).unwrap().as_str();
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
    async fn run(&self, arg: &dyn RunArg) -> Result<Box<dyn Scope>, Box<dyn Error>> {
        let mut context = arg.context().clone();
        context.insert("idx".to_string(), Value::Null);
        context.insert("item".to_string(), Value::Null);

        let args = arg.args_with(&context)?;
        trace!("{}", args);
        let array = args["arr"].as_array().ok_or(err!("103", "missing arr"))?;

        let mut map_val_vec = Vec::with_capacity(array.len());
        for (index, item) in array.iter().enumerate() {
            let mra = MapRunArg::new(arg, index, item.clone());
            let val = self.map_action.run(&mra).await?;
            map_val_vec.push(val.as_value().clone());
        }
        Ok(Box::new(Value::Array(map_val_vec)))
    }
}
