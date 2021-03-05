use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap};
use std::rc::Rc;

use serde_json::Value;

use crate::model::point::{PointContext, PointResult};

#[derive(Debug)]
pub struct CaseContext<'c, 'd> {
    config: &'c Value,
    data: &'d BTreeMap<String,String>
}


impl <'c, 'd> CaseContext <'c, 'd>{

    pub fn new(config: &'c Value, data: &'d BTreeMap<String,String>) -> CaseContext<'c, 'd>{
        let context = CaseContext {
            config,
            data
        };

        return context;
    }



    pub fn create_point(self: &CaseContext<'c, 'd>, dynamic_context_register : Rc<RefCell<HashMap<String, Value>>>) -> Vec<PointContext<'c, 'd>>{
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
                PointContext::new(
                    self.config,
                    self.data,
                    point_id,
                    dynamic_context_register.clone()
                )
            })
            .collect();
    }



    fn get_point_vec(self: &CaseContext<'c,'d>) -> Vec<String>{
        let task_point_chain_arr = self.config["task"]["chain"].as_array().unwrap();
        let task_point_chain_vec:Vec<String> = task_point_chain_arr.iter()
            .map(|e| {
                e.as_str().map(|s|String::from(s)).unwrap()
            })
            .collect();

        return task_point_chain_vec;
    }

}

pub type CaseResult = std::result::Result<Vec<(String, PointResult)>, ()>;
