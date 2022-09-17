use chord_core::action::prelude::*;

use crate::err;

pub struct MatchCreator {}

impl MatchCreator {
    pub async fn new(_: Option<Value>) -> Result<MatchCreator, Error> {
        Ok(MatchCreator {})
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
impl Creator for MatchCreator {
    async fn create(&self, _: &dyn Arg) -> Result<Box<dyn Action>, Error> {
        Ok(Box::new(Match {}))
    }
}

#[async_trait]
impl Action for Match {
    async fn execute(&self, arg: &mut dyn Arg) -> Result<Box<dyn Scope>, Error> {
        let map = arg
            .body_raw()
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
                    .chord()
                    .creator("block")
                    .ok_or(err!("101", "missing `block` action"))?
                    .create(&arg)
                    .await?;
                return bf.execute(&mut arg).await;
            }
        }

        Ok(Box::new(Value::Null))
    }
}
