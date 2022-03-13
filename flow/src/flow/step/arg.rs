use std::fmt::{Display, Formatter};
use std::sync::Arc;

use chord_core::action::Arg;
use chord_core::action::{Context, Id};
use chord_core::action::{Error, Factory};
use chord_core::case::CaseId;
use chord_core::flow::Flow;
use chord_core::value::{Map, Value};

use crate::model::app::RenderContext;
use crate::{flow, App};

#[derive(Clone)]
pub struct IdStruct {
    step: String,
    case_id: Arc<dyn CaseId>,
}

impl Id for IdStruct {
    fn step(&self) -> &str {
        self.step.as_str()
    }

    fn case_id(&self) -> &dyn CaseId {
        self.case_id.as_ref()
    }
}

impl Display for IdStruct {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.write_str(format!("{}-{}", self.case_id, self.step).as_str())
    }
}

pub struct ArgStruct<'a, 'f> {
    app: &'a dyn App,
    flow: &'f Flow,
    context: ContextStruct,
    id: IdStruct,
    aid: String,
}

impl<'a, 'f> ArgStruct<'a, 'f> {
    pub fn new(
        app: &'a dyn App,
        flow: &'f Flow,
        context: RenderContext,
        case_id: Arc<dyn CaseId>,
        step_id: String,
    ) -> ArgStruct<'a, 'f> {
        let id = IdStruct {
            case_id,
            step: step_id,
        };

        let context = ContextStruct { ctx: context };

        let run_arg = ArgStruct {
            app,
            flow,
            context,
            id,
            aid: "".to_string(),
        };

        return run_arg;
    }

    pub fn id(self: &ArgStruct<'a, 'f>) -> &IdStruct {
        return &self.id;
    }

    pub fn aid(&mut self, aid: &str) {
        self.aid = aid.to_string();
    }

    pub fn context_mut(&mut self) -> &mut dyn Context {
        &mut self.context
    }

    pub fn flow(&self) -> &Flow {
        self.flow
    }

    pub fn app(&self) -> &dyn App {
        self.app
    }
}

impl<'a, 'f> Arg for ArgStruct<'a, 'f> {
    fn id(&self) -> &dyn Id {
        &self.id
    }

    fn context(&self) -> &dyn Context {
        &self.context
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

    fn is_static(&self, raw: &Value) -> bool {
        let mut val = raw.clone();
        let rc = RenderContext::wraps(Value::Null).unwrap();
        flow::render_value(self.app.get_handlebars(), &rc, &mut val).is_ok()
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
