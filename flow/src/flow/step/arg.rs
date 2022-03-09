use std::fmt::{Display, Formatter};
use std::sync::Arc;

use handlebars::{Handlebars, TemplateRenderError};

use chord_core::action::Error;
use chord_core::action::RunId;
use chord_core::action::{CreateArg, CreateId, RunArg};
use chord_core::case::CaseId;
use chord_core::flow::Flow;
use chord_core::task::TaskId;
use chord_core::value::{Map, Value};

use crate::flow;
use crate::model::app::RenderContext;

#[derive(Clone)]
pub struct RunIdStruct {
    step: String,
    case_id: Arc<dyn CaseId>,
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
    aid: String,
}

impl<'f, 'h, 'reg> CreateArgStruct<'f, 'h, 'reg> {
    pub fn new(
        flow: &'f Flow,
        handlebars: &'h Handlebars<'reg>,
        context: Option<Map>,
        task_id: Arc<dyn TaskId>,
        action: String,
        step_id: String,
        aid: &str,
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
            aid: aid.to_string(),
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
        &self.action
    }

    fn args_raw(&self) -> &Value {
        self.flow
            .step_action_args(self.id.step(), self.aid.as_str())
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
    aid: String,
}

impl<'f, 'h, 'reg> RunArgStruct<'f, 'h, 'reg> {
    pub fn new(
        flow: &'f Flow,
        handlebars: &'h Handlebars<'reg>,
        context: RenderContext,
        case_id: Arc<dyn CaseId>,
        step_id: String,
    ) -> RunArgStruct<'f, 'h, 'reg> {
        let id = RunIdStruct {
            case_id,
            step: step_id,
        };

        let run_arg = RunArgStruct {
            flow,
            handlebars,
            context,
            id,
            aid: "".to_string(),
        };

        return run_arg;
    }

    pub fn id(self: &RunArgStruct<'f, 'h, 'reg>) -> &RunIdStruct {
        return &self.id;
    }

    pub fn aid(&mut self, aid: &str) {
        self.aid = aid.to_string();
    }
}

impl<'f, 'h, 'reg> RunArg for RunArgStruct<'f, 'h, 'reg> {
    fn id(&self) -> &dyn RunId {
        &self.id
    }

    fn context(&mut self) -> &mut Map {
        self.context.data_mut().as_object_mut().unwrap()
    }

    fn args_raw(&self) -> &Value {
        self.flow
            .step_action_args(self.id().step(), self.aid.as_str())
    }

    fn render(&self, raw: &Value) -> Result<Value, Error> {
        let mut val = raw.clone();
        flow::render_value(self.handlebars, &self.context, &mut val)?;
        Ok(val)
    }

    fn args(&self) -> Result<Value, Error> {
        self.render(self.args_raw())
    }
}
