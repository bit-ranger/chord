use std::time::Duration;

use futures::executor::block_on;
use hirofa_utils::js_utils::{JsError, Script};
use hirofa_utils::js_utils::adapters::{JsRealmAdapter, JsValueAdapter};
use hirofa_utils::js_utils::adapters::proxies::JsProxy;
use hirofa_utils::js_utils::facades::{JsRuntimeBuilder, JsRuntimeFacade};
use hirofa_utils::js_utils::facades::values::{JsValueConvertable, JsValueFacade};
use log::info;
use quickjs_runtime::builder::QuickJsRuntimeBuilder;
use quickjs_runtime::facades::QuickJsRuntimeFacade;
use quickjs_runtime::quickjsrealmadapter::QuickJsRealmAdapter;

use chord_core::action::prelude::*;

use crate::err;

pub struct QuickjsCreator {}

impl QuickjsCreator {
    pub async fn new(_: Option<Value>) -> Result<QuickjsCreator, Error> {
        Ok(QuickjsCreator {})
    }
}

#[async_trait]
impl Creator for QuickjsCreator {
    async fn create(&self, _chord: &dyn Chord, _arg: &dyn Arg) -> Result<Box<dyn Action>, Error> {
        Ok(Box::new(Quickjs {}))
    }
}

struct Quickjs {}

#[async_trait]
impl Action for Quickjs {
    async fn execute(
        &self,
        _chord: &dyn Chord,
        arg: &mut dyn Arg,
    ) -> Result<Asset, Error> {
        let code = arg
            .args_raw()
            .as_str()
            .ok_or(err!("100", "missing quickjs"))?
            .to_string();
        let rt = QuickJsRuntimeBuilder::new().js_build();
        block_on(run_code(&rt, &code))?;
        return Ok(Asset::Value(Value::Null));
    }
}

struct HttpClient {}

async fn run_code(rt: &QuickJsRuntimeFacade, code: &str) -> Result<(), JsError> {

    // create simple proxy class with an async function
    rt.js_loop_realm_sync(None, |_rt_adapter, realm_adapter| {
        let proxy = JsProxy::new(&["chord", "action"], "Http").add_static_method(
            "newClient",
            |_rt_adapter, realm_adapter: &QuickJsRealmAdapter, args| {
                let args_str = format!("{}", &args[0].js_to_i32());
                realm_adapter.js_promise_create_resolving_async(
                    async { Ok(args_str) },
                    |realm_adapter, producer_result| {
                        realm_adapter.js_string_create(producer_result.as_str())
                    },
                )
            },
        );
        realm_adapter
            .js_proxy_install(proxy, true)
            .ok()
            .expect("could not install proxy");
    });

    rt.js_eval(
        None,
        Script::new(
            "testMyProxy.js",
            code,
        ),
    ).await?;
    Ok(())
}