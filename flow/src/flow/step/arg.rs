use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::sync::Arc;

use chord_core::action::{Arg, Combo};
use chord_core::action::{Context, Id};
use chord_core::action::{Error, Player};
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

    fn clone(&self) -> Box<dyn Id> {
        let id = Clone::clone(self);
        Box::new(id)
    }
}

impl Display for IdStruct {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.write_str(format!("{}-{}", self.case_id, self.step).as_str())
    }
}

#[derive(Clone)]
pub struct ComboStruct {
    action_map: Arc<HashMap<String, Box<dyn Player>>>,
}

impl Combo for ComboStruct {
    fn action(&self, action: &str) -> Option<&dyn Player> {
        self.action_map.get(action).map(|a| a.as_ref())
    }

    fn clone(&self) -> Box<dyn Combo> {
        let combo = Clone::clone(self);
        Box::new(combo)
    }
}

pub struct ArgStruct<'a, 'f> {
    app: &'a dyn App,
    combo: ComboStruct,
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
        let combo = ComboStruct {
            action_map: app.get_action_map().clone(),
        };

        let context = ContextStruct { ctx: context };

        let id = IdStruct {
            case_id,
            step: step_id,
        };

        let run_arg = ArgStruct {
            app,
            combo,
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

    fn combo(&self) -> &dyn Combo {
        &self.combo
    }

    fn is_static(&self, raw: &Value) -> bool {
        let mut val = raw.clone();
        let rc = RenderContext::wraps(Value::Null).unwrap();
        flow::render_value(self.app.get_handlebars(), &rc, &mut val).is_ok()
    }

    fn context_mut(&mut self) -> &mut dyn Context {
        &mut self.context
    }
}

#[derive(Clone)]
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

    fn clone(&self) -> Box<dyn Context> {
        let ctx = Clone::clone(self);
        Box::new(ctx)
    }
}
