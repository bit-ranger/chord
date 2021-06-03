use std::borrow::Borrow;

use chord_common::error::Error;
use chord_common::flow::Flow;
use chord_common::step::{RunArg, CreateArg, StepId};
use chord_common::rerr;
use chord_common::value::Json;
use handlebars::{Context, Handlebars};
use log::info;

use crate::flow::case::arg::{RenderContext, CaseIdStruct};
use std::time::Duration;
use std::sync::Arc;
use chord_common::task::TaskId;
use chord_common::case::CaseId;
use std::fmt::{Display, Formatter};
use std::rc::Rc;

#[derive(Clone)]
pub struct StepIdStruct {
    step_id: String,
    case_id: Rc<dyn CaseId>
}


impl StepId for StepIdStruct {
    fn step_id(&self) -> &str {
        self.step_id.as_str()
    }

    fn case_id(&self) -> &dyn CaseId {
        self.case_id.as_ref()
    }
}
unsafe impl Send for StepIdStruct {}
unsafe impl Sync for StepIdStruct {}

impl Display for StepIdStruct {

    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.write_str(format!("{}::{}", self.case_id, self.step_id).as_str())
    }
}


pub struct CreateArgStruct <'f, 'h, 'reg, 'r>{
    flow: &'f Flow,
    handlebars: &'h Handlebars<'reg>,
    render_context: &'r RenderContext,
    kind: String,
    id: StepIdStruct,
}
unsafe impl<'f, 'h, 'reg, 'r> Send for CreateArgStruct<'f, 'h, 'reg, 'r> {}
unsafe impl<'f, 'h, 'reg, 'r> Sync for CreateArgStruct<'f, 'h, 'reg, 'r> {}


impl<'f, 'h, 'reg, 'r> CreateArgStruct<'f, 'h, 'reg, 'r> {

    pub fn new(
        flow: &'f Flow,
        handlebars: &'h Handlebars<'reg>,
        render_context: &'r RenderContext,
        task_id: Arc<dyn TaskId>,
        kind: String,
        id: String
    ) -> CreateArgStruct<'f, 'h, 'reg, 'r> {
        let context = CreateArgStruct {
            flow,
            handlebars,
            render_context,
            kind,
            id: StepIdStruct{
                case_id: Rc::new(CaseIdStruct::new(task_id, 0)),
                step_id: id
            }
        };

        return context;
    }
}

impl<'f, 'h, 'reg, 'r> CreateArg for CreateArgStruct<'f, 'h, 'reg, 'r>{


    fn id(&self) -> &dyn StepId {
        &self.id
    }

    fn kind(&self) -> &str {
        self.kind.as_str()
    }

    fn config(&self) -> &Json {
        self.flow.step_config(self.id.step_id())
    }

    fn render(&self, text: &str) -> Result<String, Error> {
        render(self.handlebars, self.render_context, text)
    }

    fn is_task_shared(&self, text: &str) -> bool {
        if let Some(_) = text.find("{{data.") {
            return false;
        }
        if let Some(_) = text.find("{{step.") {
            return false;
        }
        if let Some(_) = text.find("{{res.") {
            return false;
        }
        return true;
    }
}


pub struct RunArgStruct<'f, 'h, 'reg, 'r> {
    flow: &'f Flow,
    handlebars: &'h Handlebars<'reg>,
    render_context: &'r RenderContext,
    id: StepIdStruct,
}

unsafe impl<'f, 'h, 'reg, 'r> Send for RunArgStruct<'f, 'h, 'reg, 'r> {}
unsafe impl<'f, 'h, 'reg, 'r> Sync for RunArgStruct<'f, 'h, 'reg, 'r> {}

impl<'f, 'h, 'reg, 'r> RunArgStruct<'f, 'h, 'reg, 'r> {
    pub fn new(
        flow: &'f Flow,
        handlebars: &'h Handlebars<'reg>,
        render_context: &'r RenderContext,
        case_id: Rc<dyn CaseId>,
        id: String,
    ) -> RunArgStruct<'f, 'h, 'reg, 'r> {
        let id  = StepIdStruct {
            case_id,
            step_id: id
        };

        let context = RunArgStruct {
            flow,
            handlebars,
            render_context,
            id
        };

        return context;
    }

    pub fn id(self: &RunArgStruct<'f, 'h, 'reg, 'r>) -> &StepIdStruct{
        return &self.id;
    }

    pub async fn meta_str(
        self: &RunArgStruct<'f, 'h, 'reg, 'r>,
        path: Vec<&str>,
    ) -> Option<String> {
        let config = self.flow.step(self.id().step_id());

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
        self.flow.step_timeout(self.id().step_id())
    }
}

impl<'f, 'h, 'reg, 'r> RunArg for RunArgStruct<'f, 'h, 'reg, 'r> {
    fn id(&self) -> &dyn StepId {
        self.id()
    }

    fn config(&self) -> &Json {
        let config = self.flow.step_config(self.id().step_id());
        return config;
    }

    fn render(&self, text: &str) -> Result<String, Error> {
        return render(self.handlebars, self.render_context, text);
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