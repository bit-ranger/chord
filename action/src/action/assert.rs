use chord_core::action::prelude::*;

use crate::err;

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
    async fn run(&self, arg: &mut dyn RunArg) -> Result<Box<dyn Scope>, Error> {
        let raw = arg.args_raw();
        let raw = raw.as_str().ok_or(err!("100", "illegal exp"))?.trim();
        let assert_tpl = format!("{{{{{cond}}}}}", cond = raw);
        let result = arg.render(&Value::String(assert_tpl))?;
        if result.eq("true") {
            Ok(Box::new(
                arg.context()
                    .iter()
                    .last()
                    .map(|(_, v)| v.clone())
                    .unwrap_or(Value::Null),
            ))
        } else {
            Err(err!("100", "fail"))
        }
    }
}
