use std::collections::HashMap;

use serde::Serialize;
use serde_json::{to_value, Value};

use crate::model::{CaseContext, CaseResult, PointContext, PointResult};
use crate::point::run_point;
use std::rc::Rc;
use std::cell::RefCell;

pub async fn run_case(context: &mut CaseContext<'_,'_>) -> CaseResult {
    let dynamic_context_register = Rc::new(RefCell::new(HashMap::new()));
    let mut point_vec: Vec<PointContext> = context.create_point(dynamic_context_register.clone());
    let mut point_result_vec = Vec::<(String, PointResult)>::new();

    for  point in point_vec.iter() {
        let result = run_point(&point).await;

        match &result {
            Ok(r) => {
                register_dynamic_context(dynamic_context_register.clone(), point.get_id(), r);
            },
            Err(_) =>  {
                break;
            }
        }

        point_result_vec.push((String::from(point.get_id()), result));
    }

    return Ok(point_result_vec);
}




pub fn register_dynamic_context(dynamic_context_register : Rc<RefCell<HashMap<String, Value>>>, name: &str, result: &Value) {
    dynamic_context_register.borrow_mut().insert(String::from(name),to_value(result).unwrap());
}