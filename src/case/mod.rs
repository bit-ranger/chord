use crate::model::{CaseContext, PointContext, CaseResult, PointResult};
use crate::point::run_point;
use serde_json::{Value, to_value};
use std::collections::HashMap;
use serde::Serialize;

pub async fn run_case(context: &mut CaseContext<'_,'_>) -> CaseResult {
    let mut point_vec: Vec<PointContext> = context.create_point();
    let mut point_result_vec = Vec::<(String, PointResult)>::new();

    for  point in point_vec.iter() {
        let result = run_point(&point).await;

        match &result {
            Ok(r) => {
                context.register_dynamic_context(point.get_id(), r);
            },
            Err(_) =>  {
                break;
            }
        }

        point_result_vec.push((String::from(point.get_id()), result));
    }

    return Ok(point_result_vec);
}


// pub fn register_dynamic_context(point_value_register: &mut HashMap<String, Value>, name: &str, result: &Value) {
//     point_value_register.insert(String::from(name),to_value(result).unwrap());
// }