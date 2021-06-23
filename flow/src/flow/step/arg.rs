use std::fmt::{Display, Formatter};
use std::sync::Arc;
use std::time::Duration;

use handlebars::Handlebars;

use chord::case::CaseId;
use chord::flow::Flow;
use chord::step::{CreateArg, RunArg, StepId};
use chord::task::TaskId;
use chord::value::{from_str, to_string, Value};
use chord::Error;

use crate::flow;
use crate::flow::case::arg::CaseIdStruct;
use crate::model::app::RenderContext;

#[derive(Clone)]
pub struct StepIdStruct {
    step_id: String,
    case_id: Arc<dyn CaseId>,
}

impl StepId for StepIdStruct {
    fn step_id(&self) -> &str {
        self.step_id.as_str()
    }

    fn case_id(&self) -> &dyn CaseId {
        self.case_id.as_ref()
    }
}

impl Display for StepIdStruct {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.write_str(format!("{}::{}", self.case_id, self.step_id).as_str())
    }
}

pub struct CreateArgStruct<'f, 'h, 'reg, 'r> {
    flow: &'f Flow,
    handlebars: &'h Handlebars<'reg>,
    render_context: &'r RenderContext,
    action: String,
    id: StepIdStruct,
}

impl<'f, 'h, 'reg, 'r> CreateArgStruct<'f, 'h, 'reg, 'r> {
    pub fn new(
        flow: &'f Flow,
        handlebars: &'h Handlebars<'reg>,
        render_context: &'r RenderContext,
        task_id: Arc<dyn TaskId>,
        action: String,
        id: String,
    ) -> CreateArgStruct<'f, 'h, 'reg, 'r> {
        let context = CreateArgStruct {
            flow,
            handlebars,
            render_context,
            action,
            id: StepIdStruct {
                case_id: Arc::new(CaseIdStruct::new(
                    task_id,
                    "create".into(),
                    Arc::new("create".into()),
                )),
                step_id: id,
            },
        };

        return context;
    }
}

impl<'f, 'h, 'reg, 'r> CreateArg for CreateArgStruct<'f, 'h, 'reg, 'r> {
    fn id(&self) -> &dyn StepId {
        &self.id
    }

    fn action(&self) -> &str {
        self.action.as_str()
    }

    fn config(&self) -> &Value {
        self.flow.step_config(self.id.step_id())
    }

    fn render_str(&self, text: &str) -> Result<String, Error> {
        flow::render(self.handlebars, self.render_context, text)
    }

    fn is_task_shared(&self, text: &str) -> bool {
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
    id: StepIdStruct,
}

impl<'f, 'h, 'reg, 'r> RunArgStruct<'f, 'h, 'reg, 'r> {
    pub fn new(
        flow: &'f Flow,
        handlebars: &'h Handlebars<'reg>,
        render_context: &'r RenderContext,
        case_id: Arc<dyn CaseId>,
        id: String,
    ) -> RunArgStruct<'f, 'h, 'reg, 'r> {
        let id = StepIdStruct {
            case_id,
            step_id: id,
        };

        let context = RunArgStruct {
            flow,
            handlebars,
            render_context,
            id,
        };

        return context;
    }

    pub fn id(self: &RunArgStruct<'f, 'h, 'reg, 'r>) -> &StepIdStruct {
        return &self.id;
    }

    pub fn assert(&self) -> Option<&str> {
        self.flow.step_assert(self.id().step_id())
    }

    pub fn timeout(&self) -> Duration {
        self.flow.step_timeout(self.id().step_id())
    }
}

impl<'f, 'h, 'reg, 'r> RunArg for RunArgStruct<'f, 'h, 'reg, 'r> {
    fn id(&self) -> &dyn StepId {
        self.id()
    }

    fn config(&self) -> &Value {
        let config = self.flow.step_config(self.id().step_id());
        return config;
    }

    fn render_str(&self, txt: &str) -> Result<String, Error> {
        return flow::render(self.handlebars, self.render_context, txt);
    }

    fn render_value(&self, value: &Value) -> Result<Value, Error> {
        let value_str = to_string(&value)?;
        let value_str = self.render_str(value_str.as_str())?;
        let value: Value = from_str(value_str.as_str())?;
        return Ok(value);
    }
}
