use std::borrow::Borrow;

use chord_common::error::Error;
use chord_common::flow::Flow;
use chord_common::point::PointArg;
use chord_common::rerr;
use chord_common::value::Json;
use handlebars::{Context, Handlebars};
use log::info;

use crate::flow::case::arg::RenderContext;
use std::time::Duration;

#[derive(Debug)]
pub struct PointArgStruct<'c, 'h, 'reg, 'r, 't, 'e> {
    flow: &'c Flow,
    id: String,
    handlebars: &'h Handlebars<'reg>,
    render_context: &'r RenderContext,
    case_id: usize,
    task_id: &'t str,
    exec_id: &'e str,
}

impl<'c, 'h, 'reg, 'r, 't, 'e> PointArgStruct<'c, 'h, 'reg, 'r, 't, 'e> {
    pub fn new(
        flow: &'c Flow,
        id: &str,
        handlebars: &'h Handlebars<'reg>,
        render_context: &'r RenderContext,
        case_id: usize,
        task_id: &'t str,
        exec_id: &'e str,
    ) -> PointArgStruct<'c, 'h, 'reg, 'r, 't, 'e> {
        let context = PointArgStruct {
            flow,
            id: id.to_owned(),
            handlebars,
            render_context,
            case_id,
            task_id,
            exec_id,
        };

        return context;
    }

    pub fn id(self: &PointArgStruct<'c, 'h, 'reg, 'r, 't, 'e>) -> &str {
        return self.id.as_str();
    }

    pub async fn meta_str(
        self: &PointArgStruct<'c, 'h, 'reg, 'r, 't, 'e>,
        path: Vec<&str>,
    ) -> Option<String> {
        let config = self.flow.point(self.id());

        let raw_config = path.iter().fold(config, |acc, k| acc[k].borrow());

        match raw_config.as_str() {
            Some(s) => match render(self.handlebars, self.render_context, s) {
                Ok(s) => Some(s),
                Err(_) => None,
            },
            None => None,
        }
    }

    pub fn timeout(&self) -> Duration {
        self.flow.point_timeout(self.id())
    }

    pub fn kind(&self) -> &str {
        self.flow.point_kind(self.id())
    }
}

pub async fn assert(
    handlebars: &Handlebars<'_>,
    render_context: &Context,
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

pub fn render(
    handlebars: &Handlebars<'_>,
    render_context: &Context,
    text: &str,
) -> Result<String, Error> {
    let render = handlebars.render_template_with_context(text, render_context);
    return match render {
        Ok(r) => Ok(r),
        Err(e) => rerr!("tpl", format!("{}", e)),
    };
}

impl<'c, 'h, 'reg, 'r, 't, 'e> PointArg for PointArgStruct<'c, 'h, 'reg, 'r, 't, 'e> {
    fn id(&self) -> &str {
        self.id()
    }

    fn case_id(&self) -> usize {
        self.case_id
    }

    fn task_id(&self) -> &str {
        self.task_id
    }

    fn exec_id(&self) -> &str {
        self.exec_id
    }

    fn config(&self) -> &Json {
        let config = self.flow.point_config(self.id());
        return config;
    }

    fn render(&self, text: &str) -> Result<String, Error> {
        return render(self.handlebars, self.render_context, text);
    }

    fn is_shared(&self, text: &str) -> bool {
        if let Some(_) = text.find("{{data.") {
            return false;
        }
        if let Some(_) = text.find("{{dyn.") {
            return false;
        }
        if let Some(_) = text.find("{{res.") {
            return false;
        }
        return true;
    }
}

unsafe impl<'c, 'h, 'reg, 'r, 't, 'e> Send for PointArgStruct<'c, 'h, 'reg, 'r, 't, 'e> {}

unsafe impl<'c, 'h, 'reg, 'r, 't, 'e> Sync for PointArgStruct<'c, 'h, 'reg, 'r, 't, 'e> {}
