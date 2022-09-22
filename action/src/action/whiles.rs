use chord_core::action::prelude::*;

use crate::err;

pub struct WhileCreator {}

impl WhileCreator {
    pub async fn new(_: Option<Value>) -> Result<WhileCreator, Error> {
        Ok(WhileCreator {})
    }
}

struct While {}

struct ArgStruct<'a, 'c> {
    origin: &'a mut dyn Arg,
    cond: String,
    chord: &'c dyn Chord,
}

impl<'o, 'c> Arg for ArgStruct<'o, 'c> {
    fn id(&self) -> &dyn Id {
        self.origin.id()
    }

    fn args(&self) -> Result<Value, Error> {
        self.chord.render(self.context(), self.args_raw())
    }

    fn args_raw(&self) -> &Value {
        &self.origin.args_raw()[self.cond.as_str()]
    }

    fn args_init(&self) -> Option<&Value> {
        let raw = self.args_raw();
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
}

#[async_trait]
impl Creator for WhileCreator {
    async fn create(&self, _chord: &dyn Chord, _arg: &dyn Arg) -> Result<Box<dyn Action>, Error> {
        Ok(Box::new(While {}))
    }
}

#[async_trait]
impl Action for While {
    async fn execute(&self, chord: &dyn Chord, arg: &mut dyn Arg) -> Result<Box<dyn Scope>, Error> {
        let cond_raw = arg
            .args_raw()
            .as_object()
            .ok_or(err!("100", "while must be object"))?;

        let cond_raw = cond_raw.iter().map(|(k, _v)| k.to_string()).last().unwrap();
        let cond_tpl = format!("{{{{{cond}}}}}", cond = cond_raw.trim().to_string());
        let cond = Value::String(cond_tpl);

        loop {
            let cv = chord.render(arg.context(), &cond)?;
            if cv.is_string() && cv.as_str().unwrap().eq("true") {
                let mut arg = ArgStruct {
                    origin: arg,
                    cond: cond_raw.to_string(),
                    chord,
                };
                let bf = chord
                    .creator("block")
                    .ok_or(err!("101", "missing `block` action"))?
                    .create(chord, &arg)
                    .await?;
                bf.execute(chord, &mut arg).await?;
            } else {
                break;
            }
        }

        Ok(Box::new(Value::Null))
    }
}
