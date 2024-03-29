use chord_core::action::prelude::*;


pub struct LetCreator {}

impl LetCreator {
    pub async fn new(_: Option<Value>) -> Result<LetCreator, Error> {
        Ok(LetCreator {})
    }
}

#[async_trait]
impl Creator for LetCreator {
    async fn create(&self, _chord: &dyn Chord, _arg: &dyn Arg) -> Result<Box<dyn Action>, Error> {
        Ok(Box::new(Let {}))
    }
}

struct Let {}

#[derive(Clone)]
struct ContextStruct {
    map: Map,
}

impl Context for ContextStruct {
    fn data(&self) -> &Map {
        &self.map
    }

    fn data_mut(&mut self) -> &mut Map {
        &mut self.map
    }

    fn clone(&self) -> Box<dyn Context> {
        let ctx = Clone::clone(self);
        Box::new(ctx)
    }
}

#[async_trait]
impl Action for Let {
    async fn execute(&self, chord: &dyn Chord, arg: &mut dyn Arg) -> Result<Asset, Error> {
        let mut lets = Map::new();
        if arg.args_raw().is_object() {
            let mut new_ctx = ContextStruct {
                map: arg.context().data().clone(),
            };
            for (k, v) in arg.args_raw().as_object().unwrap() {
                let rvr = chord.render(&new_ctx, v)?;
                new_ctx.data_mut().insert(k.clone(), rvr.clone());
                lets.insert(k.clone(), rvr);
            }
            Ok(Asset::Value(Value::Object(lets)))
        } else {
            Ok(Asset::Value(arg.args()?))
        }
    }
}
