use chord::action::prelude::*;
use log::info;

pub struct AssertFactory {}

impl AssertFactory {
    pub async fn new(_: Option<Value>) -> Result<AssertFactory, Error> {
        Ok(AssertFactory {})
    }
}

#[async_trait]
impl Factory for AssertFactory {
    async fn create(&self, _: &dyn CreateArg) -> Result<Box<dyn Action>, Error> {
        Ok(Box::new(Echo {}))
    }
}

struct Echo {}

#[async_trait]
impl Action for Echo {
    async fn run(&self, arg: &dyn RunArg) -> ActionValue {
        let condition = arg.args().as_str().ok_or(err!("assert", "missing args"))?;

        let template = format!(
            "{{{{#if {condition}}}}}true{{{{else}}}}false{{{{/if}}}}",
            condition = condition
        );

        let result = arg.render_str(&template);
        return match result {
            Ok(result) => {
                if result.eq("true") {
                    Ok(Value::Null)
                } else {
                    rerr!("assert", "assert fail")
                }
            }
            Err(e) => {
                info!("assert err: {} >>> {}", condition, e);
                rerr!("assert", "assert err")
            }
        };
    }
}
