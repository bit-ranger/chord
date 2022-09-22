use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::sync::Arc;

use handlebars::Handlebars;

use chord_core::action::{Arg, Chord};
use chord_core::action::{Context, Id};
use chord_core::action::{Creator, Error};
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

pub struct ChordStruct {
    creator_map: Arc<HashMap<String, Box<dyn Creator>>>,
    app: Arc<dyn App>,
}

impl ChordStruct {
    pub fn new(app: Arc<dyn App>) -> ChordStruct {
        ChordStruct {
            creator_map: app.get_creator_map().clone(),
            app,
        }
    }

    fn render(handlebars: &Handlebars, context: &dyn Context, raw: &Value) -> Result<Value, Error> {
        let mut val = raw.clone();
        let rc = RenderContext::wraps(context.data())?;
        flow::render_value(handlebars, &rc, &mut val)?;
        Ok(val)
    }
}

impl Chord for ChordStruct {
    fn creator(&self, action: &str) -> Option<&dyn Creator> {
        let creator_op = self.creator_map.get(action);
        creator_op.map(|a| a.as_ref())
    }

    fn render(&self, context: &dyn Context, raw: &Value) -> Result<Value, Error> {
        ChordStruct::render(self.app.get_handlebars(), context, raw)
    }

    fn clone(&self) -> Box<dyn Chord> {
        Box::new(ChordStruct::new(self.app.clone()))
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
        let context = ContextStruct { ctx: context };

        let id = IdStruct {
            case_id,
            step: step_id,
        };

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

    fn args(&self) -> Result<Value, Error> {
        ChordStruct::render(&self.app.get_handlebars(), &self.context, self.args_raw())
    }

    fn context_mut(&mut self) -> &mut dyn Context {
        &mut self.context
    }

    fn args_init(&self) -> Option<&Value> {
        let raw = self.args_raw();
        if let Value::Object(obj) = raw {
            obj.get("__init__")
        } else {
            None
        }
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
