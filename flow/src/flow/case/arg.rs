use std::collections::{BTreeMap, HashMap};

use handlebars::Context;

use common::value::to_json;
use common::value::Json;
use common::flow::Flow;

use crate::flow::point::arg::PointArgStruct;
use crate::model::app::AppContext;

#[derive(Debug)]
pub struct CaseArgStruct<'c, 'd> {
    flow: &'c Flow,
    data: &'d BTreeMap<String, String>,
    id: usize
}


impl<'c, 'd> CaseArgStruct<'c, 'd> {
    pub fn new(flow: &'c Flow, data: &'d BTreeMap<String, String>, id: usize) -> CaseArgStruct<'c, 'd> {
        let context = CaseArgStruct {
            flow,
            data,
            id
        };

        return context;
    }

    pub fn create_render_context(self: &CaseArgStruct<'c, 'd>) -> RenderContext{
        let mut render_data: HashMap<&str, Json> = HashMap::new();
        let config_def = self.flow.data()["task"]["def"].as_object();
        match config_def {
            Some(def) => {
                render_data.insert("def", to_json(def).unwrap());
            }
            None => {}
        }
        render_data.insert("data", to_json(self.data).unwrap());
        render_data.insert("dyn", to_json(HashMap::<String, Json>::new()).unwrap());
        return Context::wraps(render_data).unwrap();
    }


    pub fn create_point<'h, 'reg, 'app, 'r>(self: &CaseArgStruct<'c, 'd>,
                                            point_id: &str,
                                            app_context: &'app dyn AppContext,
                                            render_context: &'r RenderContext

    ) -> Option<PointArgStruct<'c, 'd, 'h, 'reg, 'r>>
        where 'app: 'h, 'app: 'reg
    {
        let _ = self.flow.data()["point"][point_id].as_object()?;

        Some(PointArgStruct::new(
            self.flow,
            self.data,
            point_id,
            app_context.get_handlebars(),
            render_context))
    }
    
    pub fn point_id_vec(self: &CaseArgStruct<'c, 'd>) -> Vec<String> {
        self.flow.point_id_vec()
    }


    pub fn id(&self) -> usize {
        self.id
    }
}



pub type RenderContext = Context;
