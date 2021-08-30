use std::cell::RefCell;

use async_std::sync::Arc;
use async_std::task_local;
use handlebars::Handlebars;

use chord::action::Factory;
use chord::err;
use chord::input::FlowParse;
use chord::Error;
pub use task::arg::TaskIdSimple;
pub use task::TaskRunner;

use crate::model::app::{FlowApp, FlowAppStruct, RenderContext};
use chord::value::{Map, Value};

mod case;
mod step;
mod task;

task_local! {
    pub static CTX_ID: RefCell<String> = RefCell::new(String::new());
}

pub async fn context_create(
    action_factory: Box<dyn Factory>,
    flow_parse: Box<dyn FlowParse>,
) -> Arc<dyn FlowApp> {
    Arc::new(FlowAppStruct::<'_>::new(action_factory, flow_parse))
}

fn render(
    handlebars: &Handlebars<'_>,
    render_ctx: &RenderContext,
    text: &str,
) -> Result<String, Error> {
    let render = handlebars.render_template_with_context(text, render_ctx);
    return match render {
        Ok(r) => Ok(r),
        Err(e) => Err(err!("tpl", format!("{}", e))),
    };
}

fn render_ref(val: &Value, ref_from: &Value) -> Result<Value, Error> {
    return match val {
        Value::Object(map) => {
            if map.contains_key("$ref") {
                if map["$ref"].is_string() {
                    let ref_path = map["$ref"].as_str().unwrap();
                    let path: Vec<&str> = ref_path.split(".").collect();
                    let mut ref_val = ref_from;
                    for p in path {
                        ref_val = &ref_val[p];
                    }
                    Ok(ref_val.clone())
                } else {
                    Err(err!("001", "invalid args $ref"))
                }
            } else {
                let mut render_val = Map::new();
                for (k, v) in map {
                    render_val.insert(k.to_string(), render_ref(v, ref_from)?);
                }
                Ok(Value::Object(render_val))
            }
        }
        Value::Array(arr) => {
            let mut arr_val: Vec<Value> = Vec::with_capacity(arr.len());
            for a in arr {
                arr_val.push(render_ref(a, ref_from)?);
            }
            Ok(Value::Array(arr_val))
        }
        _ => Ok(val.clone()),
    };
}
