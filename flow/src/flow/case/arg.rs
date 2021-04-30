use handlebars::Context;

use chord_common::value::{to_json, Map};
use chord_common::value::Json;
use chord_common::flow::Flow;

use crate::flow::point::arg::PointArgStruct;
use crate::model::app::AppContext;
use async_std::sync::Arc;

#[derive(Debug)]
pub struct CaseArgStruct {
    id: usize,
    flow: Arc<Flow>,
    data: Json,
    point_id_vec: Vec<String>,
    context_ext:  Arc<Vec<(String, Json)>>
}


impl CaseArgStruct{
    pub fn new(id: usize,
               flow: Arc<Flow>,
               data: Json,
               point_id_vec: Vec<String>,
               context_ext: Arc<Vec<(String, Json)>>
    ) -> CaseArgStruct {
        let context = CaseArgStruct {
            id,
            flow,
            data,
            point_id_vec,
            context_ext
        };

        return context;
    }

    pub fn create_render_context(self: &CaseArgStruct) -> RenderContext{
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

        for (k,v) in self.context_ext.iter(){
            render_data.insert(k.clone(), v.clone());
        }

        return Context::wraps(render_data).unwrap();
    }


    pub fn create_point_arg<'app, 'h, 'reg, 'r>(self: &CaseArgStruct,
                                                point_id: &str,
                                                app_ctx: &'app dyn AppContext,
                                                render_ctx: &'r RenderContext

    ) -> Option<PointArgStruct<'_, 'h, 'reg, 'r>>
        where 'app: 'h, 'app: 'reg
    {
        let _ = self.flow.point(point_id).as_object()?;

        Some(PointArgStruct::new(
            self.flow.as_ref(),
            point_id,
            app_ctx.get_handlebars(),
            render_ctx))
    }
    
    pub fn point_id_vec(self: &CaseArgStruct) -> &Vec<String> {
        &self.point_id_vec
    }


    pub fn id(&self) -> usize {
        self.id
    }
}



pub type RenderContext = Context;
