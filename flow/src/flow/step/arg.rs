use std::fmt::{Display, Formatter};
use std::sync::Arc;
use std::time::Duration;

use handlebars::{Handlebars, TemplateRenderError};

use chord_core::action::Error;
use chord_core::action::RunId;
use chord_core::action::{CreateArg, CreateId, RunArg};
use chord_core::case::CaseId;
use chord_core::flow::{Flow, Then};
use chord_core::task::TaskId;
use chord_core::value::{Map, Value};

use crate::flow;
use crate::model::app::RenderContext;

#[derive(Clone)]
pub struct RunIdStruct {
    step: String,
    case_id: Arc<dyn CaseId>,
}

impl RunIdStruct {
    pub fn new(step: String, case_id: Arc<dyn CaseId>) -> RunIdStruct {
        RunIdStruct { step, case_id }
    }
}

impl RunId for RunIdStruct {
    fn step(&self) -> &str {
        self.step.as_str()
    }

    fn case_id(&self) -> &dyn CaseId {
        self.case_id.as_ref()
    }
}

impl Display for RunIdStruct {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.write_str(format!("{}-{}", self.case_id, self.step).as_str())
    }
}

#[derive(Clone)]
pub struct CreateIdStruct {
    step: String,
    task_id: Arc<dyn TaskId>,
}

impl CreateId for CreateIdStruct {
    fn step(&self) -> &str {
        self.step.as_str()
    }

    fn task_id(&self) -> &dyn TaskId {
        self.task_id.as_ref()
    }
}

impl Display for CreateIdStruct {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.write_str(format!("{}-{}", self.task_id, self.step).as_str())
    }
}

pub struct CreateArgStruct<'f, 'h, 'reg> {
    flow: &'f Flow,
    handlebars: &'h Handlebars<'reg>,
    context: RenderContext,
    action: String,
    id: CreateIdStruct,
}

impl<'f, 'h, 'reg> CreateArgStruct<'f, 'h, 'reg> {
    pub fn new(
        flow: &'f Flow,
        handlebars: &'h Handlebars<'reg>,
        context: Option<Map>,
        task_id: Arc<dyn TaskId>,
        action: String,
        step_id: String,
    ) -> CreateArgStruct<'f, 'h, 'reg> {
        let id = CreateIdStruct {
            task_id,
            step: step_id,
        };
        let context = match context {
            Some(lv) => RenderContext::wraps(lv),
            None => RenderContext::wraps(Map::new()),
        }
        .unwrap();
        let arg = CreateArgStruct {
            flow,
            handlebars,
            context,
            action,
            id,
        };
        return arg;
    }

    fn render_str(&self, text: &str) -> Result<Value, TemplateRenderError> {
        return flow::render_str(self.handlebars, &self.context, text);
    }
}

impl<'f, 'h, 'reg> CreateArg for CreateArgStruct<'f, 'h, 'reg> {
    fn id(&self) -> &dyn CreateId {
        &self.id
    }

    fn action(&self) -> &str {
        self.action.as_str()
    }

    fn args_raw(&self) -> &Value {
        self.flow.step_exec_args(self.id.step())
    }

    fn render_str(&self, text: &str) -> Result<Value, Error> {
        Ok(self.render_str(text)?)
    }

    fn is_static(&self, text: &str) -> bool {
        //handlebars.set_strict_mode(true);
        self.render_str(text).is_ok()
    }
}

pub struct RunArgStruct<'f, 'h, 'reg> {
    flow: &'f Flow,
    handlebars: &'h Handlebars<'reg>,
    context: RenderContext,
    id: RunIdStruct,
}

impl<'f, 'h, 'reg> RunArgStruct<'f, 'h, 'reg> {
    pub fn new(
        flow: &'f Flow,
        handlebars: &'h Handlebars<'reg>,
        context: Option<Map>,
        case_id: Arc<dyn CaseId>,
        step_id: String,
    ) -> RunArgStruct<'f, 'h, 'reg> {
        let id = RunIdStruct {
            case_id,
            step: step_id,
        };
        let context = match context {
            Some(lv) => RenderContext::wraps(lv),
            None => RenderContext::wraps(Map::new()),
        }
        .unwrap();
        let run_arg = RunArgStruct {
            flow,
            handlebars,
            context,
            id,
        };

        return run_arg;
    }

    pub fn id(self: &RunArgStruct<'f, 'h, 'reg>) -> &RunIdStruct {
        return &self.id;
    }

    pub fn assert(&self) -> Option<&str> {
        self.flow.step_assert(self.id().step())
    }

    pub fn timeout(&self) -> Duration {
        self.flow.step_spec_timeout(self.id().step())
    }

    pub fn catch_err(&self) -> bool {
        self.flow.step_spec_catch_err(self.id().step())
    }

    pub fn then(&self) -> Option<Vec<Then>> {
        self.flow.step_then(self.id().step())
    }

    pub fn context_mut(&mut self) -> &mut Map {
        self.context.data_mut().as_object_mut().unwrap()
    }

    pub fn render_str(&self, txt: &str) -> Result<Value, TemplateRenderError> {
        self.render_str_with(txt, &self.context)
    }

    fn render_str_with(
        &self,
        txt: &str,
        render_context: &RenderContext,
    ) -> Result<Value, TemplateRenderError> {
        return flow::render_str(self.handlebars, render_context, txt);
    }

    fn render_object_with(
        &self,
        raw: &Map,
        render_context: &RenderContext,
    ) -> Result<Map, TemplateRenderError> {
        let mut result = raw.clone();
        for (_, v) in result.iter_mut() {
            flow::render_value(self.handlebars, render_context, v)?;
        }
        Ok(result)
    }

    pub fn render_object(&self, raw: &Map) -> Result<Map, TemplateRenderError> {
        self.render_object_with(raw, &self.context)
    }
}

impl<'f, 'h, 'reg> RunArg for RunArgStruct<'f, 'h, 'reg> {
    fn id(&self) -> &dyn RunId {
        &self.id
    }

    fn context(&self) -> &Map {
        &self.context.data().as_object().unwrap()
    }

    fn timeout(&self) -> Duration {
        self.timeout()
    }

    fn args(&self) -> Result<Value, Error> {
        self.args_with(self.context.data().as_object().unwrap())
    }

    fn args_with(&self, context: &Map) -> Result<Value, Error> {
        let args_raw = self.flow.step_exec_args(self.id().step());
        let mut args_val = args_raw.clone();
        let ctx = RenderContext::wraps(context)?;
        flow::render_value(self.handlebars, &ctx, &mut args_val)?;
        return Ok(args_val);
    }
}
