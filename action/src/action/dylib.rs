use std::sync::Arc;

use dynamic_reload::{DynamicReload, Lib, PlatformName, Search, Symbol};

use chord::action::prelude::*;

pub struct DylibFactory {}

impl DylibFactory {
    pub async fn new(_: Option<Value>) -> Result<DylibFactory, Error> {
        Ok(DylibFactory {})
    }
}

#[async_trait]
impl Factory for DylibFactory {
    async fn create(&self, arg: &dyn CreateArg) -> Result<Box<dyn Action>, Error> {
        let dir = arg.args_raw()["dir"]
            .as_str()
            .ok_or(err!("100", "missing dir"))?;

        let mut reload_handler = DynamicReload::new(Some(vec![dir]), None, Search::Default);

        let lib = reload_handler.add_library("chord_dylib", PlatformName::Yes)?;

        Ok(Box::new(Dylib { lib }))
    }
}

struct Dylib {
    lib: Arc<Lib>,
}

#[async_trait]
impl Action for Dylib {
    async fn run(&self, arg: &dyn RunArg) -> Result<Box<dyn Scope>, Error> {
        let action_run: Symbol<fn(args: &str) -> Result<String, Error>> =
            unsafe { self.lib.lib.get(b"run")? };
        let mut ar = Map::new();
        ar.insert("id".to_string(), Value::String(arg.id().to_string()));
        ar.insert("args".to_string(), Value::Object(arg.args()?));
        ar.insert("context".to_string(), Value::Object(arg.context().clone()));
        ar.insert(
            "timeout".to_string(),
            Value::Number(Number::from(arg.timeout().as_secs())),
        );
        let ar = Value::Object(ar).to_string();
        let av: String = action_run(ar.as_str())?;
        let av: Value = from_str(av.as_str())?;
        Ok(Box::new(av))
    }
}
