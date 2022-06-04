use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::sync::Arc;

use dynamic_reload::{DynamicReload, Lib, PlatformName, Search, Symbol};

use chord_core::action::prelude::*;

use crate::err;

pub struct CdylibAction {
    lib_dir: String,
}

impl CdylibAction {
    pub async fn new(config: Option<Value>) -> Result<CdylibAction, Error> {
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

        Ok(CdylibAction { lib_dir })
    }
}

#[async_trait]
impl Action for CdylibAction {
    async fn player(&self, arg: &dyn Arg) -> Result<Box<dyn Player>, Error> {
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
impl Player for Cdylib {
    async fn play(&self, arg: &mut dyn Arg) -> Result<Box<dyn Scope>, Error> {
        let action_run: Symbol<fn(args: *const c_char) -> *mut c_char> =
            unsafe { self.lib.lib.get(b"run")? };
        let mut ar = Map::new();
        ar.insert("id".to_string(), Value::String(arg.id().to_string()));
        ar.insert("args".to_string(), arg.args()?);
        ar.insert(
            "context".to_string(),
            Value::Object(arg.context().data().clone()),
        );
        let ar = Value::Object(ar).to_string();
        let ar = CString::new(ar)?;
        let av: *mut c_char = action_run(ar.as_ptr());
        let av = unsafe { CStr::from_ptr(av) };
        let av: Value = from_str(av.to_str()?)?;
        Ok(Box::new(av))
    }
}
