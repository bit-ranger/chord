use chord_core::action::prelude::*;
use chord_core::action::{Context, Id};

use crate::err;

pub struct MatchFactory {}

impl MatchFactory {
    pub async fn new(_: Option<Value>) -> Result<MatchFactory, Error> {
        Ok(MatchFactory {})
    }
}

struct Match {}

struct ArgStruct<'a> {
    origin: &'a mut dyn Arg,
    cond: String,
}

impl<'o> Arg for ArgStruct<'o> {
    fn id(&self) -> &dyn Id {
        self.origin.id()
    }

    fn args(&self) -> Result<Value, Error> {
        self.render(self.context(), self.args_raw())
    }

    fn args_raw(&self) -> &Value {
        &self.origin.args_raw()[self.cond.as_str()]
    }

    fn context(&self) -> &dyn Context {
        self.origin.context()
    }

    fn context_mut(&mut self) -> &mut dyn Context {
        self.origin.context_mut()
    }

    fn render(&self, context: &dyn Context, raw: &Value) -> Result<Value, Error> {
        self.origin.render(context, raw)
    }

    fn factory(&self, action: &str) -> Option<&dyn Factory> {
        self.origin.factory(action)
    }

    fn is_static(&self, raw: &Value) -> bool {
        self.origin.is_static(raw)
    }
}

#[async_trait]
impl Factory for MatchFactory {
    async fn create(&self, _: &dyn Arg) -> Result<Box<dyn Action>, Error> {
        Ok(Box::new(Match {}))
    }
}

#[async_trait]
impl Action for Match {
    async fn run(&self, arg: &mut dyn Arg) -> Result<Box<dyn Scope>, Error> {
        let map = arg
            .args_raw()
            .as_object()
            .ok_or(err!("100", "match must be a object"))?;

        let cond_raw_vec: Vec<String> = map.iter().map(|(k, _v)| k.to_string()).collect();

        for cond_raw in cond_raw_vec {
            let cond_tpl = format!("{{{{{cond}}}}}", cond = cond_raw.trim().to_string());
            let cond = Value::String(cond_tpl);
            let cv = arg.render(arg.context(), &cond)?;
            if cv.is_string() && cv.as_str().unwrap().eq("true") {
                let mut arg = ArgStruct {
                    origin: arg,
                    cond: cond_raw.to_string(),
                };
                let bf = arg
                    .factory("block")
                    .ok_or(err!("101", "missing `block` action"))?
                    .create(&arg)
                    .await?;
                return bf.run(&mut arg).await;
            }
        }

        Ok(Box::new(Value::Null))
    }
}
