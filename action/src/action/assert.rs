
use chord_core::action::prelude::*;

use crate::err;

pub struct AssertCreator {}

impl AssertCreator {
    pub async fn new(_: Option<Value>) -> Result<AssertCreator, Error> {
        Ok(AssertCreator {})
    }
}

#[async_trait]
impl Creator for AssertCreator {
    async fn create(&self, _chord: &dyn Chord, _arg: &dyn Arg) -> Result<Box<dyn Action>, Error> {
        Ok(Box::new(Assert {}))
    }
}

struct Assert {}

#[async_trait]
impl Action for Assert {
    async fn explain(&self, _chord: &dyn Chord, arg: &dyn Arg) -> Result<Value, Error> {
        let raw = arg.args_raw();
        let raw = raw.as_str().ok_or(err!("100", "illegal assert"))?.trim();
        Ok(Value::String(raw.to_string()))
    }

    async fn execute(&self, chord: &dyn Chord, arg: &mut dyn Arg) -> Result<Asset, Error> {
        let raw = arg.args_raw();
        let raw = raw.as_str().ok_or(err!("100", "illegal assert"))?.trim();
        let assert_tpl = format!("{{{{{cond}}}}}", cond = raw);
        let ctx = arg.context();
        let result = chord.render(ctx, &Value::String(assert_tpl))?;
        if result.eq("true") {
            Ok(Asset::Value(Value::Bool(true)))
        } else {
            Err(err!("100", "false"))
        }
    }
}
