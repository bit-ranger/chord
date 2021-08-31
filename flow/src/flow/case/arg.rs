use std::fmt::{Display, Formatter};

use async_std::sync::Arc;

use chord::action::Action;
use chord::case::CaseId;
use chord::collection::TailDropVec;
use chord::flow::Flow;
use chord::task::TaskId;
use chord::value::{to_value, Map};

use crate::flow;
use crate::flow::render_ref;
use crate::flow::step::arg::RunArgStruct;
use crate::model::app::FlowApp;
use crate::model::app::RenderContext;
use chord::input::FlowParse;
use chord::step::StepState;
use chord::value::{from_str, to_string, Value};
use chord::{err, Error};
use handlebars::Handlebars;

#[derive(Clone)]
pub struct CaseIdStruct {
    task_id: Arc<dyn TaskId>,
    exec_id: Arc<String>,
    case: String,
}

impl CaseIdStruct {
    pub fn new(task_id: Arc<dyn TaskId>, case_id: String, exec_id: Arc<String>) -> CaseIdStruct {
        CaseIdStruct {
            task_id,
            exec_id,
            case: case_id,
        }
    }
}

impl CaseId for CaseIdStruct {
    fn case(&self) -> &str {
        self.case.as_str()
    }

    fn exec_id(&self) -> &str {
        self.exec_id.as_str()
    }

    fn task_id(&self) -> &dyn TaskId {
        self.task_id.as_ref()
    }
}

impl Display for CaseIdStruct {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.write_str(format!("{}-{}-{}", self.task_id, self.exec_id, self.case).as_str())
    }
}

pub struct CaseArgStruct {
    flow: Arc<Flow>,
    step_vec: Arc<TailDropVec<(String, Box<dyn Action>)>>,
    id: Arc<CaseIdStruct>,
    data: Value,
    render_ctx: RenderContext,
}

impl CaseArgStruct {
    pub fn new(
        flow: Arc<Flow>,
        step_vec: Arc<TailDropVec<(String, Box<dyn Action>)>>,
        data: Value,
        pre_ctx: Option<Arc<Value>>,
        task_id: Arc<dyn TaskId>,
        case_id: String,
        case_exec_id: Arc<String>,
    ) -> CaseArgStruct {
        let id = Arc::new(CaseIdStruct::new(task_id, case_id, case_exec_id));

        let mut render_data: Map = Map::new();
        if let Some(def) = flow.def() {
            render_data.insert(String::from("def"), to_value(def).unwrap());
        }
        render_data.insert(String::from("case"), data.clone());
        render_data.insert(String::from("step"), Value::Object(Map::new()));
        if let Some(pre_ctx) = pre_ctx.as_ref() {
            render_data.insert(String::from("pre"), pre_ctx.as_ref().clone());
        }
        let render_ctx = RenderContext::wraps(render_data).unwrap();
        return CaseArgStruct {
            flow,
            step_vec,
            id,
            data,
            render_ctx,
        };
    }

    pub fn step_vec(self: &CaseArgStruct) -> Arc<TailDropVec<(String, Box<dyn Action>)>> {
        self.step_vec.clone()
    }

    pub fn step_arg_create<'app, 'h, 'reg>(
        self: &CaseArgStruct,
        step_id: &str,
        flow_app: &'app dyn FlowApp,
    ) -> Result<RunArgStruct<'_, 'h, 'reg>, Error>
    where
        'app: 'h,
        'app: 'reg,
    {
        let let_raw = self.flow.step_let(step_id);
        let let_value = render_let_with(
            flow_app.get_handlebars(),
            flow_app.get_flow_parse(),
            &self.render_ctx,
            let_raw,
        )?;

        let render_ctx = RenderContext::wraps(let_value)?;
        RunArgStruct::new(
            self.flow.as_ref(),
            flow_app.get_handlebars(),
            render_ctx,
            self.id.clone(),
            step_id.to_owned(),
        )
    }

    pub fn id(&self) -> Arc<CaseIdStruct> {
        self.id.clone()
    }

    pub fn take_data(self) -> Value {
        self.data
    }

    pub async fn step_ok_register(&mut self, sid: &str, state: &StepState) {
        match state {
            StepState::Ok(scope) => {
                if let Value::Object(reg) = self.render_ctx.data_mut() {
                    reg["step"][sid]["value"] = scope.as_value().clone();
                }
            }
            _ => {}
        }
    }
}

fn render_let_with(
    handlebars: &Handlebars,
    flow_parse: &dyn FlowParse,
    render_ctx: &RenderContext,
    let_raw: &Value,
) -> Result<Value, Error> {
    if let_raw.is_null() {
        return Ok(Value::Null);
    }
    if let Value::String(txt) = let_raw {
        let value_str = flow::render(handlebars, render_ctx, txt.as_str())?;
        let value = flow_parse
            .parse_str(value_str.as_str())
            .map_err(|_| err!("001", format!("invalid let {}", value_str)))?;
        if value.is_object() {
            Ok(value)
        } else {
            Err(err!("001", "invalid let"))
        }
    } else if let Value::Object(map) = let_raw {
        let value_str = to_string(map)?;
        let value_str = flow::render(handlebars, render_ctx, value_str.as_str())?;
        let value: Value = from_str(value_str.as_str())
            .map_err(|_| err!("001", format!("invalid let {}", value_str)))?;
        if value.is_object() {
            let value = render_ref(&value, render_ctx.data())?;
            if value.is_object() {
                Ok(value)
            } else {
                Err(err!("001", "invalid let"))
            }
        } else {
            Err(err!("001", "invalid let"))
        }
    } else {
        Err(err!("001", "invalid let"))
    }
}
