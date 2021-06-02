use handlebars::Context;

use chord_common::flow::Flow;
use chord_common::value::Json;
use chord_common::value::{to_json, Map};

use crate::flow::point::arg::PointArgStruct;
use crate::model::app::FlowContext;
use async_std::sync::Arc;
use chord_common::point::PointRunner;

pub struct CaseArgStruct {
    id: usize,
    flow: Arc<Flow>,
    point_runner_vec: Arc<Vec<(String, Box<dyn PointRunner>)>>,
    data: Json,
    render_ctx_ext: Arc<Vec<(String, Json)>>,
    task_id: Arc<String>,
    exec_id: Arc<String>,
}

impl CaseArgStruct {
    pub fn new(
        id: usize,
        flow: Arc<Flow>,
        point_runner_vec: Arc<Vec<(String, Box<dyn PointRunner>)>>,
        data: Json,
        render_ctx_ext: Arc<Vec<(String, Json)>>,
        task_id: Arc<String>,
        exec_id: Arc<String>,
    ) -> CaseArgStruct {
        let context = CaseArgStruct {
            id,
            flow,
            point_runner_vec,
            data,
            render_ctx_ext,
            task_id,
            exec_id,
        };

        return context;
    }

    pub fn create_render_context(self: &CaseArgStruct) -> RenderContext {
        let mut render_data: Map = Map::new();
        let config_def = self.flow.task_def();
        match config_def {
            Some(def) => {
                render_data.insert(String::from("def"), to_json(def).unwrap());
            }
            None => {}
        }
        render_data.insert(String::from("data"), self.data.clone());
        render_data.insert(String::from("dyn"), Json::Object(Map::new()));
        render_data.insert(String::from("res"), Json::Null);

        for (k, v) in self.render_ctx_ext.iter() {
            render_data.insert(k.clone(), v.clone());
        }

        return Context::wraps(render_data).unwrap();
    }

    pub fn point_arg_create<'app, 'h, 'reg, 'r>(
        self: &CaseArgStruct,
        point_id: &str,
        app_ctx: &'app dyn FlowContext,
        render_ctx: &'r RenderContext,
    ) -> Option<PointArgStruct<'_, 'h, 'reg, 'r, '_, '_>>
    where
        'app: 'h,
        'app: 'reg,
    {
        let _ = self.flow.point(point_id).as_object()?;

        Some(PointArgStruct::new(
            self.flow.as_ref(),
            point_id.to_owned(),
            app_ctx.get_handlebars(),
            render_ctx,
            self.id,
            self.task_id.as_str(),
            self.exec_id.as_str(),
        ))
    }

    pub fn point_runner_vec(self: &CaseArgStruct) -> &Vec<(String, Box<dyn PointRunner>)> {
        self.point_runner_vec.as_ref()
    }

    pub fn id(&self) -> usize {
        self.id
    }
}

pub type RenderContext = Context;
