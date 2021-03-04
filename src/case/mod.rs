use crate::model::{CaseContext, PointContext, CaseResult, PointResult};
use crate::point::run_point;
use serde_json::{Value, to_value};
use std::collections::HashMap;
use serde::Serialize;

pub async fn run_case(context: &CaseContext<'_>) -> CaseResult {
    let mut point_vec: Vec<PointContext> = context.create_point();


    for mut point in point_vec.iter() {
        let result = run_point(&point).await;

        match result {
            Ok(r) => {
                context.register_value(point.get_id(), to_value(&r).unwrap());
            },
            Err(_) =>  {
                break;
            }
        }

    }

    return Ok();
}

//
// pub fn register_value(point_value_register: &mut Vec<(String, PointResult)>, name: String, result: PointResult) {
//     point_value_register.push((name, result));
// }