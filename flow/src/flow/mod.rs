use std::collections::HashMap;
use std::mem::replace;
use std::sync::Arc;

use handlebars::{Handlebars, RenderError};
use log::trace;

use chord_core::action::prelude::Map;
use chord_core::action::Creator;
use chord_core::future::task::task_local;
use chord_core::value::{from_str, Value};
pub use task::arg::TaskIdSimple;
pub use task::TaskRunner;

use crate::model::app::{App, AppStruct, RenderContext};

mod case;
mod step;
mod task;

task_local! {
    pub static CTX_ID: String;
}

pub async fn app_create(creator_map: HashMap<String, Box<dyn Creator>>) -> Arc<dyn App> {
    Arc::new(AppStruct::<'_>::new(creator_map))
}

fn render_str(
    handlebars: &Handlebars<'_>,
    render_ctx: &RenderContext,
    text: &str,
) -> Result<Value, RenderError> {
    if text.starts_with("{{") && text.ends_with("}}") {
        let text_inner_trim = &text[2..text.len() - 2].trim();
        if !text_inner_trim.contains("{{") && !text_inner_trim.contains("}}") {
            let value = if text_inner_trim.starts_with("num ") {
                let rv = handlebars.render_template_with_context(text, render_ctx)?;
                Value::Number(
                    from_str(rv.as_str()).map_err(|_| RenderError::new("invalid arg of num"))?,
                )
            } else if text_inner_trim.starts_with("bool ") {
                let rv = handlebars.render_template_with_context(text, render_ctx)?;
                Value::Bool(
                    from_str(rv.as_str()).map_err(|_| RenderError::new("invalid arg of bool"))?,
                )
            } else if text_inner_trim.starts_with("obj ") {
                let real_text = format!("{}str ({}) {}", "{{", text_inner_trim, "}}");
                trace!("obj real text: {}", real_text);
                let rv = handlebars.render_template_with_context(real_text.as_str(), render_ctx)?;
                Value::Object(
                    from_str(rv.as_str()).map_err(|_| RenderError::new("invalid arg of obj"))?,
                )
            } else if text_inner_trim.starts_with("arr ") {
                let real_text = format!("{}str ({}) {}", "{{", text_inner_trim, "}}");
                trace!("arr real text: {}", real_text);
                let rv = handlebars.render_template_with_context(real_text.as_str(), render_ctx)?;
                Value::Array(
                    from_str(rv.as_str()).map_err(|_| RenderError::new("invalid arg of arr"))?,
                )
            } else if text_inner_trim.starts_with("json ") {
                let real_text = format!("{}str ({}) {}", "{{", text_inner_trim, "}}");
                trace!("json real text: {}", real_text);
                let rv = handlebars.render_template_with_context(real_text.as_str(), render_ctx)?;
                let value: Value =
                    from_str(rv.as_str()).map_err(|_| RenderError::new("invalid arg of json"))?;
                value
            } else {
                let rv = handlebars.render_template_with_context(text, render_ctx)?;
                Value::String(rv)
            };
            return Ok(value);
        }
    }

    let rv = handlebars.render_template_with_context(text, render_ctx)?;
    Ok(Value::String(rv))
}

fn render_value(
    handlebars: &Handlebars,
    render_ctx: &RenderContext,
    value: &mut Value,
) -> Result<(), RenderError> {
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

fn assign_by_render(
    handlebars: &Handlebars,
    render_ctx: &RenderContext,
    assign_raw: &Map,
    discard_on_err: bool,
) -> Result<Map, RenderError> {
    let mut assign_value = assign_raw.clone();
    let mut new_render_ctx = render_ctx.clone();
    for (k, v) in assign_value.iter_mut() {
        let rvr = render_value(handlebars, &new_render_ctx, v);
        if rvr.is_ok() {
            if let Value::Object(m) = new_render_ctx.data_mut() {
                m.insert(k.clone(), v.clone());
            }
        } else {
            if discard_on_err {
                if let Value::Object(m) = new_render_ctx.data_mut() {
                    m.remove(k);
                }
            } else {
                rvr?;
            }
        }
    }

    Ok(assign_value)
}
