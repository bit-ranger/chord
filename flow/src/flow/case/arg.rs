use handlebars::Context;

use chord_common::flow::Flow;
use chord_common::value::Json;
use chord_common::value::{to_json, Map};

use crate::flow::step::arg::RunArgStruct;
use crate::model::app::FlowContext;
use async_std::sync::Arc;
use chord_common::case::CaseId;
use chord_common::step::StepRunner;
use chord_common::task::TaskId;
use std::fmt::{Display, Formatter};
use std::rc::Rc;

#[derive(Clone)]
pub struct CaseIdStruct {
    task_id: Arc<dyn TaskId>,
    case_id: usize,
}

impl CaseIdStruct {
    pub fn new(task_id: Arc<dyn TaskId>, case_id: usize) -> CaseIdStruct {
        CaseIdStruct { task_id, case_id }
    }
}

impl CaseId for CaseIdStruct {
    fn case_id(&self) -> usize {
        self.case_id
    }

    fn task_id(&self) -> &dyn TaskId {
        self.task_id.as_ref()
    }
}
unsafe impl Send for CaseIdStruct {}
unsafe impl Sync for CaseIdStruct {}

impl Display for CaseIdStruct {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.write_str(format!("{}::{}", self.task_id, self.case_id).as_str())
    }
}

pub struct CaseArgStruct {
    flow: Arc<Flow>,
    step_runner_vec: Arc<Vec<(String, Box<dyn StepRunner>)>>,
    data: Json,
    pre_ctx: Arc<Json>,
    id: Rc<CaseIdStruct>,
}
unsafe impl Send for CaseArgStruct {}
unsafe impl Sync for CaseArgStruct {}

impl CaseArgStruct {
    pub fn new(
        flow: Arc<Flow>,
        step_runner_vec: Arc<Vec<(String, Box<dyn StepRunner>)>>,
        data: Json,
        pre_ctx: Arc<Json>,
        task_id: Arc<dyn TaskId>,
        case_id: usize,
    ) -> CaseArgStruct {
        let id = Rc::new(CaseIdStruct::new(task_id, case_id));

        let context = CaseArgStruct {
            flow,
            step_runner_vec,
            data,
            pre_ctx,
            id,
        };

        return context;
    }

    pub fn create_render_context(self: &CaseArgStruct) -> RenderContext {
        let mut render_data: Map = Map::new();
        let config_def = self.flow.def();
        match config_def {
            Some(def) => {
                render_data.insert(String::from("def"), to_json(def).unwrap());
            }
            None => {}
        }
        render_data.insert(String::from("data"), self.data.clone());
        render_data.insert(String::from("step"), Json::Object(Map::new()));
        render_data.insert(String::from("curr"), Json::Null);
        render_data.insert(String::from("pre"), self.pre_ctx.as_ref().clone());

        return Context::wraps(render_data).unwrap();
    }

    pub fn step_arg_create<'app, 'h, 'reg, 'r>(
        self: &CaseArgStruct,
        step_id: &str,
        flow_ctx: &'app dyn FlowContext,
        render_ctx: &'r RenderContext,
    ) -> Option<RunArgStruct<'_, 'h, 'reg, 'r>>
    where
        'app: 'h,
        'app: 'reg,
    {
        let _ = self.flow.step(step_id).as_object()?;

        Some(RunArgStruct::new(
            self.flow.as_ref(),
            flow_ctx.get_handlebars(),
            render_ctx,
            self.id.clone(),
            step_id.to_owned(),
        ))
    }

    pub fn step_runner_vec(self: &CaseArgStruct) -> &Vec<(String, Box<dyn StepRunner>)> {
        self.step_runner_vec.as_ref()
    }

    pub fn id(&self) -> Rc<CaseIdStruct> {
        self.id.clone()
    }
}

pub type RenderContext = Context;
