use std::cell::RefCell;
use std::mem::replace;

use async_std::sync::Arc;
use async_std::task_local;
use handlebars::TemplateRenderError::RenderError as Re;
use handlebars::{Handlebars, RenderError, TemplateRenderError};
use log::trace;

use chord::action::prelude::Map;
use chord::action::Factory;
use chord::err;
use chord::value::{from_str, Value};
use chord::Error;
pub use task::arg::TaskIdSimple;
pub use task::TaskRunner;

use crate::model::app::{FlowApp, FlowAppStruct, RenderContext};

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
) -> Result<Value, TemplateRenderError> {
    if text.starts_with("{{") && text.ends_with("}}") {
        let text_inner_trim = &text[2..text.len() - 2].trim();
        if !text_inner_trim.contains("{{") && !text_inner_trim.contains("}}") {
            let value = if text_inner_trim.starts_with("num ") {
                let rv = handlebars.render_template_with_context(text, render_ctx)?;
                Value::Number(
                    from_str(rv.as_str())
                        .map_err(|_| Re(RenderError::new("invalid arg of num")))?,
                )
            } else if text_inner_trim.starts_with("bool ") {
                let rv = handlebars.render_template_with_context(text, render_ctx)?;
                Value::Bool(
                    from_str(rv.as_str())
                        .map_err(|_| Re(RenderError::new("invalid arg of bool")))?,
                )
            } else if text_inner_trim.starts_with("obj ") {
                let real_text = format!("{}str ({}) {}", "{{", text_inner_trim, "}}");
                trace!("obj real text: {}", real_text);
                let rv = handlebars.render_template_with_context(real_text.as_str(), render_ctx)?;
                Value::Object(
                    from_str(rv.as_str())
                        .map_err(|e| Re(RenderError::new("invalid arg of obj")))?,
                )
            } else if text_inner_trim.starts_with("arr ") {
                let real_text = format!("{}str ({}) {}", "{{", text_inner_trim, "}}");
                trace!("arr real text: {}", real_text);
                let rv = handlebars.render_template_with_context(real_text.as_str(), render_ctx)?;
                Value::Array(
                    from_str(rv.as_str())
                        .map_err(|e| Re(RenderError::new("invalid arg of arr")))?,
                )
            } else if text_inner_trim.starts_with("json ") {
                let real_text = format!("{}str ({}) {}", "{{", text_inner_trim, "}}");
                trace!("json real text: {}", real_text);
                let rv = handlebars.render_template_with_context(real_text.as_str(), render_ctx)?;
                let value: Value = from_str(rv.as_str())
                    .map_err(|e| Re(RenderError::new("invalid arg of json")))?;
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
) -> Result<(), TemplateRenderError> {
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

fn render_assign_object(
    handlebars: &Handlebars,
    render_ctx: &RenderContext,
    assign_raw: &Map,
    discard_on_err: bool,
) -> Result<Map, TemplateRenderError> {
    let mut assign_value = assign_raw.clone();
    let mut let_render_ctx = render_ctx.clone();
    let mut discard_keys = Vec::with_capacity(assign_raw.len());
    for (k, v) in assign_value.iter_mut() {
        let rvr = render_value(handlebars, &let_render_ctx, v);
        if rvr.is_ok() {
            if let Value::Object(m) = let_render_ctx.data_mut() {
                m.insert(k.clone(), v.clone());
            }
        } else {
            if discard_on_err {
                discard_keys.push(k.clone());
            } else {
                rvr?;
            }
        }
    }
    for k in discard_keys {
        assign_value.remove(&k);
    }

    Ok(assign_value)
}
