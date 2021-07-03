use std::cell::RefCell;

use async_std::sync::Arc;
use async_std::task_local;

use chord::action::Factory;
use chord::err;
use chord::Error;
pub use task::arg::TaskIdSimple;
pub use task::TaskRunner;

use crate::model::app::{Context, FlowContextStruct, RenderContext};
use handlebars::Handlebars;
use log::info;

mod case;
mod step;
mod task;

task_local! {
    pub static CTX_ID: RefCell<String> = RefCell::new(String::new());
}

pub async fn context_create(action_factory: Box<dyn Factory>) -> Arc<dyn Context> {
    Arc::new(FlowContextStruct::<'_>::new(action_factory))
}

pub fn render(
    handlebars: &Handlebars<'_>,
    render_context: &RenderContext,
    text: &str,
) -> Result<String, Error> {
    let render = handlebars.render_template_with_context(text, render_context);
    return match render {
        Ok(r) => Ok(r),
        Err(e) => Err(err!("tpl", format!("{}", e))),
    };
}

pub async fn assert(
    handlebars: &Handlebars<'_>,
    render_context: &RenderContext,
    condition: &str,
) -> bool {
    let template = format!(
        "{{{{#if {condition}}}}}true{{{{else}}}}false{{{{/if}}}}",
        condition = condition
    );

    let result = render(handlebars, render_context, &template);
    match result {
        Ok(result) => {
            if result.eq("true") {
                true
            } else {
                false
            }
        }
        Err(e) => {
            info!("assert failure: {} >>> {}", condition, e);
            false
        }
    }
}
