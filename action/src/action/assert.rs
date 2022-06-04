use chord_core::action::prelude::*;

use crate::err;

pub struct AssertAction {}

impl AssertAction {
    pub async fn new(_: Option<Value>) -> Result<AssertAction, Error> {
        Ok(AssertAction {})
    }
}

#[async_trait]
impl Action for AssertAction {
    async fn player(&self, _: &dyn Arg) -> Result<Box<dyn Player>, Error> {
        Ok(Box::new(Assert {}))
    }
}

struct Assert {}

#[async_trait]
impl Player for Assert {
    async fn explain(&self, arg: &dyn Arg) -> Result<Value, Error> {
        let raw = arg.args_raw();
        let raw = raw.as_str().ok_or(err!("100", "illegal assert"))?.trim();
        Ok(Value::String(raw.to_string()))
    }

    async fn play(&self, arg: &mut dyn Arg) -> Result<Box<dyn Scope>, Error> {
        let raw = arg.args_raw();
        let raw = raw.as_str().ok_or(err!("100", "illegal assert"))?.trim();
        let assert_tpl = format!("{{{{{cond}}}}}", cond = raw);
        let ctx = arg.context();
        let result = arg.render(ctx, &Value::String(assert_tpl))?;
        if result.eq("true") {
            Ok(Box::new(Value::Bool(true)))
        } else {
            Err(err!("100", "false"))
        }
    }
}
