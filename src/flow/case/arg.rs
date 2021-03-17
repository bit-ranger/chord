use std::collections::{BTreeMap, HashMap};

use handlebars::Context;
use serde_json::to_value;

use crate::model::app::AppContext;
use crate::model::value::Json;
use crate::flow::point::arg::PointArgStruct;

#[derive(Debug)]
pub struct CaseArgStruct<'c, 'd> {
    config: &'c Json,
    data: &'d BTreeMap<String, String>,
    id: usize
}


impl<'c, 'd> CaseArgStruct<'c, 'd> {
    pub fn new(config: &'c Json, data: &'d BTreeMap<String, String>, id: usize) -> CaseArgStruct<'c, 'd> {
        let context = CaseArgStruct {
            config,
            data,
            id
        };

        return context;
    }

    pub fn create_render_context(self: &CaseArgStruct<'c, 'd>) -> RenderContext{
        let mut render_data: HashMap<&str, Json> = HashMap::new();
        let config_def = self.config["task"]["def"].as_object();
        match config_def {
            Some(def) => {
                render_data.insert("def", to_value(def).unwrap());
            }
            None => {}
        }
        render_data.insert("data", to_value(self.data).unwrap());
        render_data.insert("dyn", to_value(HashMap::<String, Json>::new()).unwrap());
        return Context::wraps(render_data).unwrap();
    }


    pub fn create_point<'h, 'reg, 'app, 'r>(self: &CaseArgStruct<'c, 'd>,
                                            point_id: &str,
                                            app_context: &'app dyn AppContext,
                                            render_context: &'r RenderContext

    ) -> Option<PointArgStruct<'c, 'd, 'h, 'reg, 'r>>
        where 'app: 'h, 'app: 'reg
    {
        let _ = self.config["point"][point_id].as_object()?;

        Some(PointArgStruct::new(
            self.config,
            self.data,
            point_id,
            app_context.get_handlebars(),
            render_context))
    }


    pub fn get_point_id_vec(self: &CaseArgStruct<'c, 'd>) -> Vec<String> {
        let task_point_chain_arr = self.config["task"]["chain"].as_array().unwrap();
        let task_point_chain_vec: Vec<String> = task_point_chain_arr.iter()
            .map(|e| {
                e.as_str().map(|s| String::from(s)).unwrap()
            })
            .collect();

        return task_point_chain_vec;
    }


    pub fn id(&self) -> usize {
        self.id
    }
}



pub type RenderContext = Context;
