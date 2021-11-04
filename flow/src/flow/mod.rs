use std::cell::RefCell;
use std::mem::replace;

use async_std::sync::Arc;
use async_std::task_local;
use handlebars::Handlebars;

use chord::action::Factory;
use chord::err;
use chord::Error;
pub use task::arg::TaskIdSimple;
pub use task::TaskRunner;

use crate::model::app::{FlowApp, FlowAppStruct, RenderContext};
use async_std::path::Path;
use chord::value::{from_str, Value};

mod case;
mod step;
mod task;

task_local! {
    pub static CTX_ID: RefCell<String> = RefCell::new(String::new());
}

pub async fn context_create(action_factory: Box<dyn Factory>) -> Arc<dyn FlowApp> {
    Arc::new(FlowAppStruct::<'_>::new(action_factory))
}

fn render_str(
    handlebars: &Handlebars<'_>,
    render_ctx: &RenderContext,
    task_dir: &Path,
    text: &str,
) -> Result<Value, Error> {
    let rv = handlebars
        .render_template_with_context(text, render_ctx)
        .map_err(|e| err!("tpl", format!("{}", e)))?;
    render_dollar_str(render_ctx.data(), task_dir, rv)
}

fn render_value(
    handlebars: &Handlebars,
    render_ctx: &RenderContext,
    task_dir: &Path,
    value: &mut Value,
) -> Result<(), Error> {
    match value {
        Value::String(v) => {
            let vr = render_str(handlebars, render_ctx, task_dir, v)?;
            let _ = replace(value, vr);
            Ok(())
        }
        Value::Object(v) => {
            for (_, v) in v.iter_mut() {
                render_value(handlebars, render_ctx, task_dir, v)?;
            }
            Ok(())
        }
        Value::Array(v) => {
            for i in v {
                render_value(handlebars, render_ctx, task_dir, i)?;
            }
            Ok(())
        }
        Value::Null => Ok(()),
        Value::Bool(_) => Ok(()),
        Value::Number(_) => Ok(()),
    }
}

fn render_dollar_str(context: &Value, task_dir: &Path, text: String) -> Result<Value, Error> {
    let value = if text.starts_with("$num:") {
        Value::Number(from_str(&text[5..]).map_err(|_| err!("001", "invalid args $num"))?)
    } else if text.starts_with("$bool:") {
        Value::Bool(from_str(&text[6..]).map_err(|_| err!("001", "invalid args $bool"))?)
    } else if text.starts_with("$obj:") {
        Value::Object(
            from_str(&text[5..]).map_err(|e| err!("001", format!("invalid args $obj, {}", e)))?,
        )
    } else if text.starts_with("$arr:") {
        Value::Array(
            from_str(&text[5..]).map_err(|e| err!("001", format!("invalid args $arr, {}", e)))?,
        )
    } else if text.starts_with("$ref:") {
        let ref_path = &text[5..];
        let path: Vec<&str> = ref_path.split(".").collect();
        let mut ref_val = context;
        for p in path {
            ref_val = &ref_val[p];
        }
        ref_val.clone()
    } else if text.starts_with("$file:") {
        let ref_path = &text[6..];
        let mut task_path = task_dir.to_path_buf();
        task_path.push(ref_path);
        let string = std::fs::read_to_string(task_path)?;
        Value::String(string)
    } else {
        Value::String(text)
    };
    Ok(value)
}
