use std::time::Duration;

use hirofa_utils::js_utils::{JsError, Script};
use hirofa_utils::js_utils::facades::{JsRuntimeBuilder, JsRuntimeFacade};
use hirofa_utils::js_utils::modules::NativeModuleLoader;
use quickjs_runtime::builder::QuickJsRuntimeBuilder;
use quickjs_runtime::facades::QuickJsRuntimeFacade;
use quickjs_runtime::quickjs_utils::primitives::from_i32;
use quickjs_runtime::quickjsrealmadapter::QuickJsRealmAdapter;
use quickjs_runtime::valueref::JSValueRef;

use chord_core::action::prelude::*;

use crate::err;

pub struct QuickjsPlayer {}

impl QuickjsPlayer {
    pub async fn new(_: Option<Value>) -> Result<QuickjsPlayer, Error> {
        Ok(QuickjsPlayer {})
    }
}

#[async_trait]
impl Player for QuickjsPlayer {
    async fn action(&self, _: &dyn Arg) -> Result<Box<dyn Action>, Error> {
        Ok(Box::new(Quickjs {}))
    }
}

struct Quickjs {}

#[async_trait]
impl Action for Quickjs {

    async fn run(&self, arg: &mut dyn Arg) -> Result<Box<dyn Scope>, Error> {
        let combo = arg.combo().clone();
        let context = arg.context().data().clone();
        let id = arg.id().clone();
        let code = arg
            .args_raw()
            .as_str()
            .ok_or(err!("100", "missing quickjs"))?
            .to_string();
        futures::executor::block_on(
            execute(id, code, combo, context)
        )

    }
}




async fn execute(
    id: Box<dyn Id>,
    code: String,
    combo: Box<dyn Combo>,
    context: Map,
) -> Result<Box<dyn Scope>, Error> {

    struct ChordModuleLoader {}
    impl NativeModuleLoader<QuickJsRealmAdapter> for ChordModuleLoader {
        fn has_module(&self, _q_ctx: &QuickJsRealmAdapter, module_name: &str) -> bool {
            let has_module = module_name.eq("chord");
            println!("has_module {} {}", module_name, has_module);
            has_module
        }

        fn get_module_export_names(&self, realm: &QuickJsRealmAdapter, module_name: &str) -> Vec<&str> {
            println!("get_module_export_names {}", module_name);
            vec!["chordVal", ]
        }

        fn get_module_exports(&self, ctx: &QuickJsRealmAdapter, module_name: &str) -> Vec<(&str, JSValueRef)> {
            println!("get_module_exports {}", module_name);

            let js_val = from_i32(1470);
            // let js_func = functions::new_function_q(
            //     ctx,
            //     "someFunc", |_q_ctx, _this, _args| {
            //         return Ok(from_i32(432));
            //     }, 0)
            //     .ok().unwrap();
            // let js_class = Proxy::new()
            //     .name("SomeClass")
            //     .static_method("doIt", |_q_ctx, _args|{
            //         return Ok(from_i32(185));
            //     })
            //     .install(ctx, false)
            //     .ok().unwrap();

            vec![("chordVal", js_val)]
        }
    }

    let rt = QuickJsRuntimeFacade::builder()
        .gc_interval(Duration::from_secs(1))
        .max_stack_size(128 * 1024)
        .memory_limit(1024000)
        .native_module_loader(Box::new(ChordModuleLoader{}))
        .build();

    // let result = rt.js_eval_module(
    //     Some("chord"),
    //     Script::new("chord/core.es", "import {chordVal} from \"chord\";")).await;
    //
    // if let Err(e) = result {
    //     return Err(err!("101", format!("{}", e)));
    // }

    let result = rt.js_eval(
        None,
        Script::new(
            "chord/core.es",
            "import('chord').then((chordVal) => { return 233; })")
    ).await;

    match result {
        Ok(v) => {
            Ok(Box::new(Value::Number(Number::from(v.get_i32()))))
        }
        Err(e) => Err(err!("101", format!("{}", e))),
    }


}