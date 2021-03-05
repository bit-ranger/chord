use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap};
use std::rc::Rc;

use serde_json::{to_value};
use crate::model::Json;

use crate::model::point::{PointContextStruct, PointResult};
use handlebars::Context;
use crate::model::Error;

#[derive(Debug)]
pub struct CaseContextStruct<'c, 'd> {
    config: &'c Json,
    data: &'d BTreeMap<String,String>
}


impl <'c, 'd> CaseContextStruct<'c, 'd>{

    pub fn new(config: &'c Json, data: &'d BTreeMap<String,String>) -> CaseContextStruct<'c, 'd>{
        let context = CaseContextStruct {
            config,
            data
        };

        return context;
    }



    pub fn create_point(self: &CaseContextStruct<'c, 'd>) -> Vec<PointContextStruct<'c, 'd>>{
        let mut render_data:HashMap<&str, Json> = HashMap::new();
        let config_def = self.config["task"]["def"].as_object();
        match config_def{
            Some(def) => {
                render_data.insert("def", to_value(def).unwrap());
            },
            None => {}
        }
        render_data.insert("data", to_value(self.data).unwrap());
        render_data.insert("dyn", to_value(HashMap::<String, Json>::new()).unwrap());

        let render_context  = Rc::new(RefCell::new(Context::wraps(render_data).unwrap()));

        return self.get_point_vec()
            .into_iter()
            .filter(|point_id| {
                let none = self.config["point"][point_id].as_object().is_none();
                if none {
                    panic!("missing point config {}", point_id);
                } else {
                    return true;
                }
            })
            .map(|point_id| {
                PointContextStruct::new(
                    self.config,
                    self.data,
                    point_id,
                    render_context.clone()
                )
            })
            .collect();
    }



    fn get_point_vec(self: &CaseContextStruct<'c,'d>) -> Vec<String>{
        let task_point_chain_arr = self.config["task"]["chain"].as_array().unwrap();
        let task_point_chain_vec:Vec<String> = task_point_chain_arr.iter()
            .map(|e| {
                e.as_str().map(|s|String::from(s)).unwrap()
            })
            .collect();

        return task_point_chain_vec;
    }

}

pub type CaseResult = std::result::Result<Vec<(String, PointResult)>, Error>;
