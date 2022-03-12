use std::fmt::{Display, Formatter};
use std::sync::Arc;

use handlebars::TemplateRenderError;

use chord_core::action::{Context, RunId};
use chord_core::action::{CreateArg, CreateId, RunArg};
use chord_core::action::{Error, Factory};
use chord_core::case::CaseId;
use chord_core::flow::Flow;
use chord_core::task::TaskId;
use chord_core::value::{Map, Value};

use crate::model::app::RenderContext;
use crate::{flow, FlowApp};

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

pub struct CreateArgStruct<'a, 'f> {
    app: &'a dyn FlowApp,
    flow: &'f Flow,
    context: ContextStruct,
    action: String,
    id: CreateIdStruct,
    aid: String,
}

impl<'a, 'f> CreateArgStruct<'a, 'f> {
    pub fn new(
        app: &'a dyn FlowApp,
        flow: &'f Flow,
        context: Option<Map>,
        task_id: Arc<dyn TaskId>,
        action: String,
        step_id: String,
        aid: &str,
    ) -> CreateArgStruct<'a, 'f> {
        let id = CreateIdStruct {
            task_id,
            step: step_id,
        };
        let context = match context {
            Some(lv) => RenderContext::wraps(lv),
            None => RenderContext::wraps(Map::new()),
        }
        .unwrap();

        let context = ContextStruct { ctx: context };

        let arg = CreateArgStruct {
            app,
            flow,
            context,
            action,
            id,
            aid: aid.to_string(),
        };
        return arg;
    }

    fn render_str(&self, text: &str) -> Result<Value, TemplateRenderError> {
        return flow::render_str(self.app.get_handlebars(), self.context.as_ref(), text);
    }
}

impl<'a, 'f> CreateArg for CreateArgStruct<'a, 'f> {
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

    fn factory(&self, action: &str) -> Option<&dyn Factory> {
        self.app.get_action_factory(action)
    }
}

pub struct RunArgStruct<'a, 'f> {
    app: &'a dyn FlowApp,
    flow: &'f Flow,
    context: ContextStruct,
    id: RunIdStruct,
    aid: String,
}

impl<'a, 'f> RunArgStruct<'a, 'f> {
    pub fn new(
        app: &'a dyn FlowApp,
        flow: &'f Flow,
        context: RenderContext,
        case_id: Arc<dyn CaseId>,
        step_id: String,
    ) -> RunArgStruct<'a, 'f> {
        let id = RunIdStruct {
            case_id,
            step: step_id,
        };

        let context = ContextStruct { ctx: context };

        let run_arg = RunArgStruct {
            app,
            flow,
            context,
            id,
            aid: "".to_string(),
        };

        return run_arg;
    }

    pub fn id(self: &RunArgStruct<'a, 'f>) -> &RunIdStruct {
        return &self.id;
    }

    pub fn aid(&mut self, aid: &str) {
        self.aid = aid.to_string();
    }
}

impl<'a, 'f> RunArg for RunArgStruct<'a, 'f> {
    fn id(&self) -> &dyn RunId {
        &self.id
    }

    fn context(&self) -> &dyn Context {
        &self.context
    }

    fn context_mut(&mut self) -> &mut dyn Context {
        &mut self.context
    }

    fn args_raw(&self) -> &Value {
        self.flow
            .step_action_args(self.id().step(), self.aid.as_str())
    }

    fn render(&self, context: &dyn Context, raw: &Value) -> Result<Value, Error> {
        let mut val = raw.clone();
        let rc = RenderContext::wraps(context.data())?;
        flow::render_value(self.app.get_handlebars(), &rc, &mut val)?;
        Ok(val)
    }

    fn args(&self) -> Result<Value, Error> {
        self.render(&self.context, self.args_raw())
    }

    fn factory(&self, action: &str) -> Option<&dyn Factory> {
        self.app.get_action_factory(action)
    }
}

struct ContextStruct {
    ctx: RenderContext,
}

impl AsRef<RenderContext> for ContextStruct {
    fn as_ref(&self) -> &RenderContext {
        &self.ctx
    }
}

impl Context for ContextStruct {
    fn data(&self) -> &Map {
        self.ctx.data().as_object().unwrap()
    }

    fn data_mut(&mut self) -> &mut Map {
        self.ctx.data_mut().as_object_mut().unwrap()
    }
}
