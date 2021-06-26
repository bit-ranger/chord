use std::fmt::{Display, Formatter};

use async_std::sync::Arc;

use chord::action::Action;
use chord::case::CaseId;
use chord::flow::Flow;
use chord::task::TaskId;
use chord::value::Value;
use chord::value::{to_value, Map};

use crate::flow::step::arg::RunArgStruct;
use crate::model::app::Context;
use crate::model::app::RenderContext;

#[derive(Clone)]
pub struct CaseIdStruct {
    task_id: Arc<dyn TaskId>,
    case_id: String,
    exec_id: Arc<String>,
}

impl CaseIdStruct {
    pub fn new(task_id: Arc<dyn TaskId>, case_id: String, exec_id: Arc<String>) -> CaseIdStruct {
        CaseIdStruct {
            task_id,
            case_id,
            exec_id,
        }
    }
}

impl CaseId for CaseIdStruct {
    fn id(&self) -> &str {
        self.case_id.as_str()
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
        f.write_str(format!("{}-{}-{}", self.task_id, self.exec_id, self.case_id).as_str())
    }
}

pub struct CaseArgStruct {
    flow: Arc<Flow>,
    action_vec: Arc<Vec<(String, Box<dyn Action>)>>,
    data: Value,
    pre_ctx: Arc<Value>,
    id: Arc<CaseIdStruct>,
}

impl CaseArgStruct {
    pub fn new(
        flow: Arc<Flow>,
        action_vec: Arc<Vec<(String, Box<dyn Action>)>>,
        data: Value,
        pre_ctx: Arc<Value>,
        task_id: Arc<dyn TaskId>,
        case_id: String,
        case_exec_id: Arc<String>,
    ) -> CaseArgStruct {
        let id = Arc::new(CaseIdStruct::new(task_id, case_id, case_exec_id));

        let context = CaseArgStruct {
            flow,
            action_vec,
            data,
            pre_ctx,
            id,
        };

        return context;
    }

    pub fn create_render_context(self: &CaseArgStruct) -> RenderContext {
        let mut render_data: Map = Map::new();
        let config_def = self.flow.def();
        if let Some(def) = config_def {
            render_data.insert(String::from("def"), to_value(def).unwrap());
        }
        render_data.insert(String::from("case"), self.data.clone());
        render_data.insert(String::from("step"), Value::Object(Map::new()));
        render_data.insert(String::from("curr"), Value::Null);
        render_data.insert(String::from("pre"), self.pre_ctx.as_ref().clone());

        return RenderContext::wraps(render_data).unwrap();
    }

    pub fn step_arg_create<'app, 'h, 'reg, 'r>(
        self: &CaseArgStruct,
        step_id: &str,
        flow_ctx: &'app dyn Context,
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

    pub fn action_vec(self: &CaseArgStruct) -> Arc<Vec<(String, Box<dyn Action>)>> {
        self.action_vec.clone()
    }

    pub fn id(&self) -> Arc<CaseIdStruct> {
        self.id.clone()
    }

    pub fn take_data(self) -> Value {
        self.data
    }
}
