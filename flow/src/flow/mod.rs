use std::cell::RefCell;
use std::mem::replace;

use async_std::sync::Arc;
use async_std::task_local;
use handlebars::Handlebars;

use chord::action::Factory;
use chord::err;
use chord::Error;
use log::trace;
pub use task::arg::TaskIdSimple;
pub use task::TaskRunner;

use crate::model::app::{FlowApp, FlowAppStruct, RenderContext};
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
    text: &str,
) -> Result<Value, Error> {
    if text.starts_with("{{") && text.ends_with("}}") {
        let text_inner_trim = &text[2..text.len() - 2].trim();
        if !text_inner_trim.contains("{{") && !text_inner_trim.contains("}}") {
            let value = if text_inner_trim.starts_with("num ") {
                let rv = handlebars
                    .render_template_with_context(text, render_ctx)
                    .map_err(|e| err!("tpl", format!("{}", e)))?;
                Value::Number(from_str(rv.as_str()).map_err(|_| err!("001", "invalid args num"))?)
            } else if text_inner_trim.starts_with("bool ") {
                let rv = handlebars
                    .render_template_with_context(text, render_ctx)
                    .map_err(|e| err!("tpl", format!("{}", e)))?;
                Value::Bool(from_str(rv.as_str()).map_err(|_| err!("001", "invalid args bool"))?)
            } else if text_inner_trim.starts_with("obj ") {
                let real_text = format!("{}str ({}) {}", "{{", text_inner_trim, "}}");
                trace!("obj real text: {}", real_text);
                let rv = handlebars
                    .render_template_with_context(real_text.as_str(), render_ctx)
                    .map_err(|e| err!("tpl", format!("{}", e)))?;
                Value::Object(
                    from_str(rv.as_str())
                        .map_err(|e| err!("001", format!("invalid args obj, {}", e)))?,
                )
            } else if text_inner_trim.starts_with("arr ") {
                let real_text = format!("{}str ({}) {}", "{{", text_inner_trim, "}}");
                trace!("arr real text: {}", real_text);
                let rv = handlebars
                    .render_template_with_context(real_text.as_str(), render_ctx)
                    .map_err(|e| err!("tpl", format!("{}", e)))?;
                Value::Array(
                    from_str(rv.as_str())
                        .map_err(|e| err!("001", format!("invalid args arr, {}", e)))?,
                )
            } else if text_inner_trim.starts_with("json ") {
                let real_text = format!("{}str ({}) {}", "{{", text_inner_trim, "}}");
                trace!("json real text: {}", real_text);
                let rv = handlebars
                    .render_template_with_context(real_text.as_str(), render_ctx)
                    .map_err(|e| err!("tpl", format!("{}", e)))?;
                let value: Value = from_str(rv.as_str())
                    .map_err(|e| err!("001", format!("invalid args arr, {}", e)))?;
                value
            } else {
                let rv = handlebars
                    .render_template_with_context(text, render_ctx)
                    .map_err(|e| err!("tpl", format!("{}", e)))?;
                Value::String(rv)
            };
            return Ok(value);
        }
    }

    let rv = handlebars
        .render_template_with_context(text, render_ctx)
        .map_err(|e| err!("tpl", format!("{}", e)))?;
    Ok(Value::String(rv))
}

fn render_value(
    handlebars: &Handlebars,
    render_ctx: &RenderContext,
    value: &mut Value,
) -> Result<(), Error> {
    match value {
        Value::String(v) => {
            let vr = render_str(handlebars, render_ctx, v)?;
            let _ = replace(value, vr);
            Ok(())
        }
        Value::Object(v) => {
            for (_, v) in v.iter_mut() {
                render_value(handlebars, render_ctx, v)?;
            }
            Ok(())
        }
        Value::Array(v) => {
            for i in v {
                render_value(handlebars, render_ctx, i)?;
            }
            Ok(())
        }
        Value::Null => Ok(()),
        Value::Bool(_) => Ok(()),
        Value::Number(_) => Ok(()),
    }
}
