use crate::action::CommonScope;
use chord::action::prelude::*;
use chord::action::RenderContextUpdate;
use log::{info, trace};

pub struct IterFilterFactory {}

impl IterFilterFactory {
    pub async fn new(_: Option<Value>) -> Result<IterFilterFactory, Error> {
        Ok(IterFilterFactory {})
    }
}

#[async_trait]
impl Factory for IterFilterFactory {
    async fn create(&self, _: &dyn CreateArg) -> Result<Box<dyn Action>, Error> {
        Ok(Box::new(IterFilter {}))
    }
}

struct IterFilter {}

impl IterFilter {}

async fn render_condition(condition: &str, idx: usize, item: &Value, arg: &dyn RunArg) -> bool {
    let template = format!(
        "{{{{#if {condition}}}}}true{{{{else}}}}false{{{{/if}}}}",
        condition = condition
    );

    let result = arg.render_str(
        template.as_str(),
        Some(Box::new(FilterContext {
            idx,
            item: item.clone(),
        })),
    );
    match result {
        Ok(result) => {
            if result.eq("true") {
                true
            } else {
                false
            }
        }
        Err(e) => {
            info!("render_condition failure: {} >>> {}", condition, e);
            false
        }
    }
}

struct FilterContext {
    idx: usize,
    item: Value,
}

impl RenderContextUpdate for FilterContext {
    fn update(&self, value: &mut Value) {
        value["idx"] = Value::Number(Number::from(self.idx));
        value["item"] = self.item.clone();
    }
}

struct IterFilterContext {}

impl RenderContextUpdate for IterFilterContext {
    fn update(&self, value: &mut Value) {
        if let Value::Object(data) = value {
            if !data.contains_key("idx") {
                data.insert("idx".to_string(), Value::Number(Number::from(0)));
            }
            if !data.contains_key("item") {
                data.insert("item".to_string(), Value::Null);
            }
        }
    }
}

#[async_trait]
impl Action for IterFilter {
    async fn run(&self, arg: &dyn RunArg) -> Result<Box<dyn Scope>, Error> {
        let args = arg.args(Some(Box::new(IterFilterContext {})))?;
        trace!("{}", args);
        let array = args["arr"].as_array().ok_or(err!("103", "missing arr"))?;
        let filter = args["filter"]
            .as_str()
            .ok_or(err!("103", "missing filter"))?;

        let mut value_vec = Vec::with_capacity(filter.len());
        for (idx, item) in array.iter().enumerate() {
            if render_condition(filter, idx, item, arg).await {
                value_vec.push(item.clone());
            }
        }

        Ok(Box::new(CommonScope {
            args,
            value: Value::Array(value_vec),
        }))
    }
}
