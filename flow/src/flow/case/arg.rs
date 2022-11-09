use std::fmt::{Display, Formatter};
use std::sync::Arc;

use chord_core::case::CaseId;
use chord_core::collection::TailDropVec;
use chord_core::flow::Flow;
use chord_core::step::{StepAsset, StepState};
use chord_core::task::TaskId;
use chord_core::value::Map;
use chord_core::value::Value;

use crate::flow::step::{action_asset_to_value, StepRunner};
use crate::flow::step::arg::ArgStruct;
use crate::flow::step::res::StepAssetStruct;
use crate::model::app::App;
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

    fn stage_id(&self) -> &str {
        self.stage_id.as_str()
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
    step_vec: Arc<TailDropVec<(String, StepRunner)>>,
    id: Arc<CaseIdStruct>,
    data: Value,
    render_ctx: RenderContext,
}

impl CaseArgStruct {
    pub fn new(
        flow: Arc<Flow>,
        step_vec: Arc<TailDropVec<(String, StepRunner)>>,
        data: Value,
        pre_ctx: Option<Arc<Map>>,
        def_ctx: Option<Arc<Map>>,
        task_id: Arc<dyn TaskId>,
        stage_id: Arc<String>,
        case_exec_id: Arc<String>,
        case_id: String,
    ) -> CaseArgStruct {
        let id = Arc::new(CaseIdStruct::new(task_id, stage_id, case_exec_id, case_id));

        let mut render_data: Map = Map::new();
        render_data.insert("__meta__".to_owned(), Value::Object(flow.meta().clone()));
        if let Some(def_ctx) = def_ctx {
            render_data.insert(String::from("def"), Value::Object(def_ctx.as_ref().clone()));
        }
        render_data.insert(String::from("case"), data.clone());
        if let Some(pre_ctx) = pre_ctx.as_ref() {
            render_data.insert(String::from("pre"), Value::Object(pre_ctx.as_ref().clone()));
        }
        if !render_data.contains_key("step") {
            render_data.insert(String::from("step"), Value::Object(Map::new()));
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

    pub fn step_vec(self: &CaseArgStruct) -> Arc<TailDropVec<(String, StepRunner)>> {
        self.step_vec.clone()
    }

    pub fn step_arg_create<'app>(
        self: &CaseArgStruct,
        step_id: &str,
        flow_app: &'app dyn App,
    ) -> ArgStruct<'app, '_> {
        ArgStruct::new(
            flow_app,
            self.flow.as_ref(),
            self.render_ctx.clone(),
            self.id.clone(),
            step_id.to_owned(),
        )
    }

    pub async fn step_asset_register(&mut self, sid: &str, step_asset: &StepAssetStruct) {
        if let StepState::Ok(av) = step_asset.state() {
            if let Value::Object(reg) = self.render_ctx.data_mut() {
                let mut am = Map::new();
                for a in av.iter() {
                    am.insert(a.id().to_string(), action_asset_to_value(a.as_ref()));
                }
                reg["step"][sid] = Value::Object(am);
            }
        }
    }

    pub fn id(&self) -> Arc<CaseIdStruct> {
        self.id.clone()
    }

    pub fn take_data(self) -> Value {
        self.data
    }
}
