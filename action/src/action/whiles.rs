use chord_core::action::prelude::*;

use crate::err;

pub struct WhileCreator {}

impl WhileCreator {
    pub async fn new(_: Option<Value>) -> Result<WhileCreator, Error> {
        Ok(WhileCreator {})
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

    fn body(&self) -> Result<Value, Error> {
        self.render(self.context(), self.body_raw())
    }

    fn body_raw(&self) -> &Value {
        &self.origin.body_raw()[self.cond.as_str()]
    }

    fn init(&self) -> Option<&Value> {
        let raw = self.body_raw();
        if let Value::Object(obj) = raw {
            obj.get("__init__")
        } else {
            None
        }
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

    fn chord(&self) -> &dyn Chord {
        self.origin.chord()
    }
}

#[async_trait]
impl Creator for WhileCreator {
    async fn create(&self, _: &dyn Arg) -> Result<Box<dyn Action>, Error> {
        Ok(Box::new(While {}))
    }
}

#[async_trait]
impl Action for While {
    async fn execute(&self, arg: &mut dyn Arg) -> Result<Box<dyn Scope>, Error> {
        let cond_raw = arg
            .body_raw()
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
                    .chord()
                    .creator("block")
                    .ok_or(err!("101", "missing `block` action"))?
                    .create(&arg)
                    .await?;
                bf.execute(&mut arg).await?;
            } else {
                break;
            }
        }

        Ok(Box::new(Value::Null))
    }
}
