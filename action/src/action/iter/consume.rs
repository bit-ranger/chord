use async_std::sync::Arc;
use chord::action::prelude::*;
use chord::action::{Context, CreateId, RunId};
use chord::collection::TailDropVec;
use itertools::Itertools;
use log::trace;
use std::collections::HashMap;
use std::time::Duration;

pub struct IterConsumeFactory {
    table: HashMap<String, Arc<dyn Factory>>,
}

impl IterConsumeFactory {
    pub async fn new(
        _: Option<Value>,
        table: HashMap<String, Arc<dyn Factory>>,
    ) -> Result<IterConsumeFactory, Error> {
        Ok(IterConsumeFactory { table })
    }
}

struct ConsumeCreateArg<'a> {
    step_id: String,
    iter_arg: &'a dyn CreateArg,
}

impl<'a> CreateArg for ConsumeCreateArg<'a> {
    fn id(&self) -> &dyn CreateId {
        self.iter_arg.id()
    }

    fn action(&self) -> &str {
        self.iter_arg.action()
    }

    fn args_raw(&self) -> &Value {
        &self.iter_arg.args_raw()["consume"][self.step_id.as_str()]["args"]
    }

    fn render_str(&self, text: &str, ctx: Option<Box<dyn Context>>) -> Result<String, Error> {
        self.iter_arg.render_str(text, ctx)
    }

    fn is_static(&self, text: &str) -> bool {
        self.iter_arg.is_static(text)
    }
}

struct IterConsume {
    step_vec: TailDropVec<(String, Box<dyn Action>)>,
}

struct ConsumeRunArg<'a, 'i, 'r> {
    step_id: String,
    iter_arg: &'a dyn RunArg,
    index: usize,
    index_name: String,
    item: &'i Value,
    item_name: String,
    step_value: &'r Map,
}

impl<'a, 'i, 'r> RunArg for ConsumeRunArg<'a, 'i, 'r> {
    fn id(&self) -> &dyn RunId {
        self.iter_arg.id()
    }

    fn args(&self, ctx: Option<Box<dyn Context>>) -> Result<Value, Error> {
        let map_ctx = ConsumeContext {
            upper_ctx: ctx,
            index: self.index,
            index_name: self.index_name.clone(),
            item: self.item.clone(),
            item_name: self.item_name.clone(),
            step_value: self.step_value.clone(),
        };
        Ok(
            self.iter_arg.args(Some(Box::new(map_ctx)))?["consume"][self.step_id.as_str()]["args"]
                .clone(),
        )
    }

    fn timeout(&self) -> Duration {
        self.iter_arg.timeout()
    }

    fn render_str(&self, text: &str, ctx: Option<Box<dyn Context>>) -> Result<String, Error> {
        self.iter_arg.render_str(text, ctx)
    }
}

struct ConsumeContext {
    upper_ctx: Option<Box<dyn Context>>,
    index: usize,
    index_name: String,
    item: Value,
    item_name: String,
    step_value: Map,
}

impl Context for ConsumeContext {
    fn update(&self, value: &mut Value) {
        value[self.index_name.as_str()] = Value::Number(Number::from(self.index));
        value[self.item_name.as_str()] = self.item.clone();
        for (step_id, step_val) in self.step_value.iter() {
            value["consume"][step_id]["value"] = step_val.clone();
        }
        if let Some(ctx) = self.upper_ctx.as_ref() {
            ctx.update(value);
        }
    }
}

#[async_trait]
impl Factory for IterConsumeFactory {
    async fn create(&self, arg: &dyn CreateArg) -> Result<Box<dyn Action>, Error> {
        let map = arg.args_raw()["consume"]
            .as_object()
            .ok_or(err!("101", "missing map"))?;

        let mut step_vec = Vec::with_capacity(map.len());
        for (step_id, step_obj) in map.iter() {
            let action = step_obj["action"]
                .as_str()
                .ok_or(err!("102", "missing action"))?;
            let factory = match action {
                "iter_consume" => self as &dyn Factory,
                _ => self
                    .table
                    .get(action)
                    .ok_or(err!("102", format!("unsupported action {}", action)))?
                    .as_ref(),
            };

            let map_create_arg = ConsumeCreateArg {
                iter_arg: arg,
                step_id: step_id.clone(),
            };

            let step_action = factory.create(&map_create_arg).await?;
            step_vec.push((step_id.clone(), step_action))
        }

        Ok(Box::new(IterConsume {
            step_vec: TailDropVec::from(step_vec),
        }))
    }
}

#[async_trait]
impl Action for IterConsume {
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

        for (index, item) in array.iter().enumerate() {
            let mut step_value = Map::new();
            for (step_id, step_action) in self.step_vec.iter() {
                let mra = ConsumeRunArg {
                    step_id: step_id.to_string(),
                    iter_arg: arg,
                    index,
                    index_name: index_name.to_string(),
                    item,
                    item_name: item_name.to_string(),
                    step_value: &step_value,
                };
                let val = step_action.run(&mra).await?;
                step_value.insert(step_id.to_string(), val.as_value().clone());
            }
        }
        Ok(Box::new(Value::Null))
    }
}
