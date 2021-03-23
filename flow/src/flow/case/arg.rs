use handlebars::Context;

use common::value::{to_json, Map};
use common::value::Json;
use common::flow::Flow;

use crate::flow::point::arg::PointArgStruct;
use crate::model::app::AppContext;

#[derive(Debug)]
pub struct CaseArgStruct<'c, 'd, 'p> {
    id: usize,
    flow: &'c Flow,
    data: &'d Json,
    point_id_vec: Vec<String>,
    context: &'p Vec<(String, Json)>
}


impl<'c, 'd, 'p> CaseArgStruct<'c, 'd, 'p> {
    pub fn new(id: usize,
               flow: &'c Flow,
               data: &'d Json,
               point_id_vec: Vec<String>,
               context: &'p Vec<(String, Json)>
    ) -> CaseArgStruct<'c, 'd, 'p> {
        let context = CaseArgStruct {
            id,
            flow,
            data,
            point_id_vec,
            context
        };

        return context;
    }

    pub fn create_render_context(self: &CaseArgStruct<'c, 'd, 'p>) -> RenderContext{
        let mut render_data: Map = Map::new();
        let config_def = self.flow.data()["task"]["def"].as_object();
        match config_def {
            Some(def) => {
                render_data.insert(String::from("def"), to_json(def).unwrap());
            }
            None => {}
        }
        render_data.insert(String::from("data"), self.data.clone());
        render_data.insert(String::from("dyn"), Json::Object(Map::new()));

        for (k,v) in self.context{
            render_data.insert(k.clone(), v.clone());
        }

        return Context::wraps(render_data).unwrap();
    }


    pub fn create_point<'h, 'reg, 'app, 'r>(self: &CaseArgStruct<'c, 'd, 'p>,
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
    
    pub fn point_id_vec(self: &CaseArgStruct<'c, 'd, 'p>) -> &Vec<String> {
        &self.point_id_vec
    }


    pub fn id(&self) -> usize {
        self.id
    }
}



pub type RenderContext = Context;
