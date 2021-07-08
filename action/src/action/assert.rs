use log::info;

use chord::action::prelude::*;

pub struct AssertFactory {}

impl AssertFactory {
    pub async fn new(_: Option<Value>) -> Result<AssertFactory, Error> {
        Ok(AssertFactory {})
    }
}

#[async_trait]
impl Factory for AssertFactory {
    async fn create(&self, _: &dyn CreateArg) -> Result<Box<dyn Action>, Error> {
        Ok(Box::new(Assert {}))
    }
}

struct Assert {}

#[async_trait]
impl Action for Assert {
    async fn run(&self, arg: &dyn RunArg) -> Result<Box<dyn Scope>, Error> {
        let condition = arg.args().as_str().ok_or(err!("100", "missing args"))?;

        let template = format!(
            "{{{{#if {condition}}}}}true{{{{else}}}}false{{{{/if}}}}",
            condition = condition
        );

        let result = arg.render_str(&template);
        return match result {
            Ok(result) => {
                if result.eq("true") {
                    Ok(Box::new(Value::Null))
                } else {
                    Err(err!("101", "assert fail"))
                }
            }
            Err(e) => {
                info!("assert err: {} >>> {}", condition, e);
                Err(err!("102", "assert err"))
            }
        };
    }
}
