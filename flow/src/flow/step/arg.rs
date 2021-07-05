use std::fmt::{Display, Formatter};
use std::sync::Arc;
use std::time::Duration;

use handlebars::Handlebars;

use chord::action::RunId;
use chord::action::{CreateArg, CreateId, RunArg};
use chord::case::CaseId;
use chord::flow::Flow;
use chord::task::TaskId;
use chord::value::{from_str, to_string, Value};
use chord::Error;

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

pub struct CreateArgStruct<'f, 'h, 'reg, 'r> {
    flow: &'f Flow,
    handlebars: &'h Handlebars<'reg>,
    render_context: &'r RenderContext,
    action: String,
    id: CreateIdStruct,
}

impl<'f, 'h, 'reg, 'r> CreateArgStruct<'f, 'h, 'reg, 'r> {
    pub fn new(
        flow: &'f Flow,
        handlebars: &'h Handlebars<'reg>,
        render_context: &'r RenderContext,
        task_id: Arc<dyn TaskId>,
        action: String,
        step_id: String,
    ) -> CreateArgStruct<'f, 'h, 'reg, 'r> {
        let id = CreateIdStruct {
            task_id,
            step: step_id,
        };
        let context = CreateArgStruct {
            flow,
            handlebars,
            render_context,
            action,
            id,
        };

        return context;
    }
}

impl<'f, 'h, 'reg, 'r> CreateArg for CreateArgStruct<'f, 'h, 'reg, 'r> {
    fn id(&self) -> &dyn CreateId {
        &self.id
    }

    fn action(&self) -> &str {
        self.action.as_str()
    }

    fn args(&self) -> &Value {
        self.flow.step_args(self.id.step())
    }

    fn render_str(&self, text: &str) -> Result<String, Error> {
        flow::render(self.handlebars, self.render_context, text)
    }

    fn is_shared(&self, text: &str) -> bool {
        if let Some(_) = text.find("{{data.") {
            return false;
        }
        if let Some(_) = text.find("{{step.") {
            return false;
        }
        if let Some(_) = text.find("{{curr.") {
            return false;
        }
        return true;
    }
}

pub struct RunArgStruct<'f, 'h, 'reg, 'r> {
    flow: &'f Flow,
    handlebars: &'h Handlebars<'reg>,
    render_context: &'r RenderContext,
    id: RunIdStruct,
}

impl<'f, 'h, 'reg, 'r> RunArgStruct<'f, 'h, 'reg, 'r> {
    pub fn new(
        flow: &'f Flow,
        handlebars: &'h Handlebars<'reg>,
        render_context: &'r RenderContext,
        case_id: Arc<dyn CaseId>,
        step_id: String,
    ) -> RunArgStruct<'f, 'h, 'reg, 'r> {
        let id = RunIdStruct {
            case_id,
            step: step_id,
        };

        let context = RunArgStruct {
            flow,
            handlebars,
            render_context,
            id,
        };

        return context;
    }

    pub fn id(self: &RunArgStruct<'f, 'h, 'reg, 'r>) -> &RunIdStruct {
        return &self.id;
    }

    pub fn assert(&self) -> Option<&str> {
        self.flow.step_assert(self.id().step())
    }

    pub fn timeout(&self) -> Duration {
        self.flow.step_timeout(self.id().step())
    }
}

impl<'f, 'h, 'reg, 'r> RunArg for RunArgStruct<'f, 'h, 'reg, 'r> {
    fn id(&self) -> &dyn RunId {
        &self.id
    }

    fn args(&self) -> &Value {
        self.flow.step_args(self.id().step())
    }

    fn render_str(&self, txt: &str) -> Result<String, Error> {
        return flow::render(self.handlebars, self.render_context, txt);
    }

    fn render_value(&self, value: &Value) -> Result<Value, Error> {
        if value.is_null() {
            return Ok(Value::Null);
        }
        let value_str = to_string(&value)?;
        let value_str = self.render_str(value_str.as_str())?;
        let value: Value = from_str(value_str.as_str())?;
        return Ok(value);
    }
}
