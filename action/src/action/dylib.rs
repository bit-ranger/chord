use std::sync::Arc;

use dynamic_reload::{DynamicReload, Lib, PlatformName, Search, Symbol};

use chord::action::prelude::*;

pub struct DylibFactory {
    lib_dir: String,
}

impl DylibFactory {
    pub async fn new(config: Option<Value>) -> Result<DylibFactory, Error> {
        if config.is_none() {
            return Err(err!("100", "missing action.dylib"));
        }
        let config = config.as_ref().unwrap();

        if config.is_null() {
            return Err(err!("101", "missing action.dylib"));
        }

        let lib_dir = config["dir"]
            .as_str()
            .ok_or(err!("103", "missing dylib.dir"))?
            .to_owned();

        Ok(DylibFactory { lib_dir })
    }
}

#[async_trait]
impl Factory for DylibFactory {
    async fn create(&self, arg: &dyn CreateArg) -> Result<Box<dyn Action>, Error> {
        let args_raw = Value::Object(arg.args_raw().clone());
        let lib_name = args_raw["lib"].as_str().ok_or(err!("100", "missing lib"))?;

        let mut reload_handler =
            DynamicReload::new(Some(vec![self.lib_dir.as_str()]), None, Search::Default);
        let lib = reload_handler.add_library(lib_name, PlatformName::Yes)?;

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
