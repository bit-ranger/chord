use std::fmt::{Display, Formatter};
use std::sync::Arc;
use std::time::Duration;

use handlebars::Handlebars;

use chord::action::{CreateArg, CreateId, RunArg};
use chord::action::{RenderContextUpdate, RunId};
use chord::case::CaseId;
use chord::flow::Flow;
use chord::task::TaskId;
use chord::value::{from_str, to_string, Map, Value};
use chord::{err, Error};

use crate::flow;
use crate::flow::render_dollar;
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

pub struct CreateArgStruct<'f> {
    flow: &'f Flow,
    action: String,
    id: CreateIdStruct,
}

impl<'f, 'h, 'reg, 'r> CreateArgStruct<'f> {
    pub fn new(
        flow: &'f Flow,
        _: &'h Handlebars<'reg>,
        _: &'r RenderContext,
        task_id: Arc<dyn TaskId>,
        action: String,
        step_id: String,
    ) -> CreateArgStruct<'f> {
        let id = CreateIdStruct {
            task_id,
            step: step_id,
        };
        let context = CreateArgStruct { flow, action, id };

        return context;
    }

    fn render_str(
        &self,
        text: &str,
        _: Option<Box<dyn RenderContextUpdate>>,
    ) -> Result<String, Error> {
        Ok(text.to_string())
    }
}

impl<'f> CreateArg for CreateArgStruct<'f> {
    fn id(&self) -> &dyn CreateId {
        &self.id
    }

    fn action(&self) -> &str {
        self.action.as_str()
    }

    fn args_raw(&self) -> &Value {
        self.flow.step_exec_args(self.id.step())
    }

    fn render_str(
        &self,
        text: &str,
        ctx: Option<Box<dyn RenderContextUpdate>>,
    ) -> Result<String, Error> {
        self.render_str(&text, ctx)
    }

    fn is_static(&self, text: &str) -> bool {
        if let Some(_) = text.find("{{") {
            return false;
        }
        return true;
    }
}

pub struct RunArgStruct<'f, 'h, 'reg> {
    flow: &'f Flow,
    handlebars: &'h Handlebars<'reg>,
    render_context: RenderContext,
    id: RunIdStruct,
}

impl<'f, 'h, 'reg> RunArgStruct<'f, 'h, 'reg> {
    pub fn new(
        flow: &'f Flow,
        handlebars: &'h Handlebars<'reg>,
        let_value: Value,
        case_id: Arc<dyn CaseId>,
        step_id: String,
    ) -> Result<RunArgStruct<'f, 'h, 'reg>, Error> {
        let id = RunIdStruct {
            case_id,
            step: step_id,
        };
        let render_context = if let_value.is_null() {
            RenderContext::wraps(Value::Object(Map::new()))
        } else {
            RenderContext::wraps(let_value)
        }?;
        let run_arg = RunArgStruct {
            flow,
            handlebars,
            render_context,
            id,
        };
        return Ok(run_arg);
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

    pub fn then(&self) -> Option<Vec<&Map>> {
        self.flow.step_then(self.id().step())
    }

    fn render_str_with(&self, txt: &str, render_context: &RenderContext) -> Result<String, Error> {
        return flow::render(self.handlebars, render_context, txt);
    }

    fn render_args_with(
        &self,
        args_raw: &Value,
        render_context: &RenderContext,
    ) -> Result<Value, Error> {
        if args_raw.is_null() {
            return Ok(Value::Null);
        }
        if let Value::Object(map) = args_raw {
            let value_str = to_string(map)?;
            let value_str = self.render_str_with(value_str.as_str(), render_context)?;
            let value: Value = from_str(value_str.as_str())
                .map_err(|_| err!("001", format!("invalid args {}", value_str)))?;
            if value.is_object() {
                let value = render_dollar(&value, render_context.data())?;
                if value.is_object() {
                    Ok(value)
                } else {
                    Err(err!("001", "invalid args"))
                }
            } else {
                Err(err!("001", "invalid args"))
            }
        } else {
            Err(err!("001", "invalid args"))
        }
    }

    fn render_args(
        &self,
        args_raw: &Value,
        ctx: Option<Box<dyn RenderContextUpdate>>,
    ) -> Result<Value, Error> {
        if args_raw.is_null() {
            return Ok(Value::Null);
        }
        return if ctx.is_some() {
            let ctx = ctx.unwrap();
            let mut render_context = self.render_context.clone();
            ctx.update(render_context.data_mut());
            self.render_args_with(args_raw, &render_context)
        } else {
            self.render_args_with(args_raw, &self.render_context)
        };
    }

    fn render_str(
        &self,
        text: &str,
        ctx: Option<Box<dyn RenderContextUpdate>>,
    ) -> Result<String, Error> {
        if ctx.is_some() {
            let ctx = ctx.unwrap();
            let mut render_context = self.render_context.clone();
            ctx.update(render_context.data_mut());
            self.render_str_with(text, &render_context)
        } else {
            self.render_str_with(text, &self.render_context)
        }
    }
}

impl<'f, 'h, 'reg> RunArg for RunArgStruct<'f, 'h, 'reg> {
    fn id(&self) -> &dyn RunId {
        &self.id
    }

    fn args(&self, ctx: Option<Box<dyn RenderContextUpdate>>) -> Result<Value, Error> {
        let args_raw = self.flow.step_exec_args(self.id().step());
        return self.render_args(args_raw, ctx);
    }

    fn timeout(&self) -> Duration {
        self.timeout()
    }

    fn render_str(
        &self,
        text: &str,
        ctx: Option<Box<dyn RenderContextUpdate>>,
    ) -> Result<String, Error> {
        self.render_str(&text, ctx)
    }
}
