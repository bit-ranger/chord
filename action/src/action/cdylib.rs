use std::sync::Arc;

use dynamic_reload::{DynamicReload, Lib, PlatformName, Search, Symbol};

use chord::action::prelude::*;

use crate::err;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;

pub struct CdylibFactory {
    lib_dir: String,
}

impl CdylibFactory {
    pub async fn new(config: Option<Value>) -> Result<CdylibFactory, Error> {
        if config.is_none() {
            return Err(err!("100", "missing action.cdylib"));
        }
        let config = config.as_ref().unwrap();

        if config.is_null() {
            return Err(err!("101", "missing action.cdylib"));
        }

        let lib_dir = config["dir"]
            .as_str()
            .ok_or(err!("103", "missing cdylib.dir"))?
            .to_owned();

        Ok(CdylibFactory { lib_dir })
    }
}

#[async_trait]
impl Factory for CdylibFactory {
    async fn create(&self, arg: &dyn CreateArg) -> Result<Box<dyn Action>, Error> {
        let args_raw = arg.args_raw();
        let lib_name = args_raw.as_str().ok_or(err!("100", "missing lib"))?;

        let mut reload_handler =
            DynamicReload::new(Some(vec![self.lib_dir.as_str()]), None, Search::Default);
        let lib = reload_handler.add_library(lib_name, PlatformName::Yes)?;

        Ok(Box::new(Cdylib { lib }))
    }
}

struct Cdylib {
    lib: Arc<Lib>,
}

#[async_trait]
impl Action for Cdylib {
    async fn run(&self, arg: &dyn RunArg) -> Result<Box<dyn Scope>, Error> {
        let action_run: Symbol<fn(args: *const c_char) -> *mut c_char> =
            unsafe { self.lib.lib.get(b"run")? };
        let mut ar = Map::new();
        ar.insert("id".to_string(), Value::String(arg.id().to_string()));
        ar.insert("args".to_string(), arg.args()?);
        ar.insert("context".to_string(), Value::Object(arg.context().clone()));
        ar.insert(
            "timeout".to_string(),
            Value::Number(Number::from(arg.timeout().as_secs())),
        );
        let ar = Value::Object(ar).to_string();
        let ar = CString::new(ar)?;
        let av: *mut c_char = action_run(ar.as_ptr());
        let av = unsafe { CStr::from_ptr(av) };
        println!("cdylib_example return {}", av.to_str()?);
        let av: Value = from_str(av.to_str()?)?;
        Ok(Box::new(av))
    }
}
