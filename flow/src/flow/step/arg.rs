use std::fmt::{Display, Formatter};
use std::sync::Arc;
use std::time::Duration;

use handlebars::Handlebars;

use chord::action::{Context, RunId};
use chord::action::{CreateArg, CreateId, RunArg};
use chord::case::CaseId;
use chord::flow::Flow;
use chord::task::TaskId;
use chord::value::{from_str, to_string, Map, Value};
use chord::{err, Error};

use crate::flow;
use crate::model::app::RenderContext;
use chord::input::FlowParse;

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

pub struct CreateArgStruct<'f, 'h, 'reg, 'r> {
    flow: &'f Flow,
    handlebars: &'h Handlebars<'reg>,
    render_context: &'r RenderContext,
    action: String,
    id: CreateIdStruct,
}

impl<'f, 'h, 'reg, 'r, 'p> CreateArgStruct<'f, 'h, 'reg, 'r> {
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

    fn args_raw(&self) -> &Value {
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

pub struct RunArgStruct<'f, 'h, 'reg, 'r, 'p> {
    flow: &'f Flow,
    handlebars: &'h Handlebars<'reg>,
    render_context: &'r RenderContext,
    flow_parse: &'p dyn FlowParse,
    id: RunIdStruct,
}

impl<'f, 'h, 'reg, 'r, 'p> RunArgStruct<'f, 'h, 'reg, 'r, 'p> {
    pub fn new(
        flow: &'f Flow,
        handlebars: &'h Handlebars<'reg>,
        render_context: &'r RenderContext,
        flow_parse: &'p dyn FlowParse,
        case_id: Arc<dyn CaseId>,
        step_id: String,
    ) -> Result<RunArgStruct<'f, 'h, 'reg, 'r, 'p>, Error> {
        let id = RunIdStruct {
            case_id,
            step: step_id,
        };

        let run_arg = RunArgStruct {
            flow,
            handlebars,
            render_context,
            flow_parse,
            id,
        };
        return Ok(run_arg);
    }

    pub fn id(self: &RunArgStruct<'f, 'h, 'reg, 'r, 'p>) -> &RunIdStruct {
        return &self.id;
    }

    pub fn assert(&self) -> Option<&str> {
        self.flow.step_assert(self.id().step())
    }

    pub fn timeout(&self) -> Duration {
        self.flow.step_timeout(self.id().step())
    }

    pub fn catch_err(&self) -> bool {
        self.flow.step_catch_err(self.id().step())
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
        if let Value::String(txt) = args_raw {
            let value_str = self.render_str_with(txt.as_str(), render_context)?;
            let value = self
                .flow_parse
                .parse_str(value_str.as_str())
                .map_err(|_| err!("001", format!("invalid args {}", value_str)))?;
            if value.is_object() {
                Ok(value)
            } else {
                Err(err!("001", "invalid args"))
            }
        } else if let Value::Object(map) = args_raw {
            let value_str = to_string(map)?;
            let value_str = self.render_str_with(value_str.as_str(), render_context)?;
            let value: Value = from_str(value_str.as_str())
                .map_err(|_| err!("001", format!("invalid args {}", value_str)))?;
            if value.is_object() {
                let value = self.render_ref(&value, render_context.data())?;
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

    fn render_args(&self, args_raw: &Value, ctx: Option<Box<dyn Context>>) -> Result<Value, Error> {
        if args_raw.is_null() {
            return Ok(Value::Null);
        }
        return if ctx.is_some() {
            let ctx = ctx.unwrap();
            let mut render_context = self.render_context.clone();
            ctx.update(render_context.data_mut());
            self.render_args_with(args_raw, &render_context)
        } else {
            self.render_args_with(args_raw, self.render_context)
        };
    }

    fn render_ref(&self, val: &Value, ref_from: &Value) -> Result<Value, Error> {
        return match val {
            Value::Object(map) => {
                if map.contains_key("$ref") {
                    if map["$ref"].is_string() {
                        let ref_path = map["$ref"].as_str().unwrap();
                        let path: Vec<&str> = ref_path.split(".").collect();
                        let mut ref_val = ref_from;
                        for p in path {
                            ref_val = &ref_val[p];
                        }
                        Ok(ref_val.clone())
                    } else {
                        Err(err!("001", "invalid args $ref"))
                    }
                } else {
                    let mut render_val = Map::new();
                    for (k, v) in map {
                        render_val.insert(k.to_string(), self.render_ref(v, ref_from)?);
                    }
                    Ok(Value::Object(render_val))
                }
            }
            Value::Array(arr) => {
                let mut arr_val: Vec<Value> = Vec::with_capacity(arr.len());
                for a in arr {
                    arr_val.push(self.render_ref(a, ref_from)?);
                }
                Ok(Value::Array(arr_val))
            }
            _ => Ok(val.clone()),
        };
    }
}

impl<'f, 'h, 'reg, 'r, 'p> RunArg for RunArgStruct<'f, 'h, 'reg, 'r, 'p> {
    fn id(&self) -> &dyn RunId {
        &self.id
    }

    fn args(&self, ctx: Option<Box<dyn Context>>) -> Result<Value, Error> {
        let args_raw = self.flow.step_args(self.id().step());
        return self.render_args(args_raw, ctx);
    }

    fn timeout(&self) -> Duration {
        self.timeout()
    }
}
