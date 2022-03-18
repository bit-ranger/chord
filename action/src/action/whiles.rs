use chord_core::action::prelude::*;
use chord_core::action::{Context, Id};

use crate::err;

pub struct WhileFactory {}

impl WhileFactory {
    pub async fn new(_: Option<Value>) -> Result<WhileFactory, Error> {
        Ok(WhileFactory {})
    }
}

struct While {}

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
impl Factory for WhileFactory {
    async fn create(&self, _: &dyn Arg) -> Result<Box<dyn Action>, Error> {
        Ok(Box::new(While {}))
    }
}

#[async_trait]
impl Action for While {
    async fn run(&self, arg: &mut dyn Arg) -> Result<Box<dyn Scope>, Error> {
        let cond_raw = arg
            .args_raw()
            .as_object()
            .ok_or(err!("100", "while must be object"))?;

        let cond_raw = cond_raw.iter().map(|(k, _v)| k.to_string()).last().unwrap();
        let cond_tpl = format!("{{{{{cond}}}}}", cond = cond_raw.trim().to_string());
        let cond = Value::String(cond_tpl);

        loop {
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
                bf.run(&mut arg).await?;
            } else {
                break;
            }
        }

        Ok(Box::new(Value::Null))
    }
}
