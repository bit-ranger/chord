use std::fmt::{Display, Formatter};

use async_std::sync::Arc;
use log::trace;

use chord::action::Action;
use chord::case::CaseId;
use chord::collection::TailDropVec;
use chord::flow::Flow;
use chord::step::{StepAssess, StepState};
use chord::task::TaskId;
use chord::value::Map;
use chord::value::Value;
use chord::Error;

use crate::flow;
use crate::flow::step::arg::RunArgStruct;
use crate::flow::step::res::StepAssessStruct;
use crate::model::app::FlowApp;
use crate::model::app::RenderContext;

#[derive(Clone)]
pub struct CaseIdStruct {
    task_id: Arc<dyn TaskId>,
    stage_id: Arc<String>,
    exec_id: Arc<String>,
    case: String,
}

impl CaseIdStruct {
    pub fn new(
        task_id: Arc<dyn TaskId>,
        stage_id: Arc<String>,
        exec_id: Arc<String>,
        case_id: String,
    ) -> CaseIdStruct {
        CaseIdStruct {
            task_id,
            stage_id,
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
        f.write_str(
            format!(
                "{}-{}-{}-{}",
                self.task_id, self.stage_id, self.exec_id, self.case
            )
            .as_str(),
        )
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
        pre_ctx: Option<Arc<Map>>,
        let_ctx: Option<Arc<Map>>,
        task_id: Arc<dyn TaskId>,
        stage_id: Arc<String>,
        case_exec_id: Arc<String>,
        case_id: String,
    ) -> CaseArgStruct {
        let id = Arc::new(CaseIdStruct::new(task_id, stage_id, case_exec_id, case_id));

        let mut render_data: Map = Map::new();
        render_data.insert("__meta__".to_owned(), Value::Object(flow.meta().clone()));
        if let Some(let_ctx) = let_ctx {
            render_data.insert(String::from("let"), Value::Object(let_ctx.as_ref().clone()));
        }
        if let Some(pre_ctx) = pre_ctx.as_ref() {
            render_data.insert(String::from("pre"), Value::Object(pre_ctx.as_ref().clone()));
        }
        render_data.insert(String::from("case"), data.clone());
        render_data.insert(String::from("step"), Value::Object(Map::new()));
        render_data.insert(String::from("reg"), Value::Object(Map::new()));

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
        let let_value = match let_raw {
            Some(let_raw) => {
                let let_value = flow::render_assign_object(
                    flow_app.get_handlebars(),
                    &self.render_ctx,
                    let_raw,
                )?;
                Some(let_value)
            }
            None => None,
        };

        RunArgStruct::new(
            self.flow.as_ref(),
            flow_app.get_handlebars(),
            let_value,
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

    pub async fn step_ok_register(&mut self, sid: &str, step_assess: &StepAssessStruct) {
        if let StepState::Ok(scope) = step_assess.state() {
            if let Value::Object(reg) = self.render_ctx.data_mut() {
                reg["step"][sid]["value"] = scope.as_value().clone();
                if let Some(then) = step_assess.then() {
                    if let Some(r) = then.reg() {
                        for (k, v) in r {
                            trace!("step reg {} {} {}", sid, k, v);
                            reg["reg"][k] = v.clone()
                        }
                    }
                }
            }
        }
    }
}
