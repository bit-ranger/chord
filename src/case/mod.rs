use crate::model::{CaseContext, PointContext, CaseResult, PointResult};
use crate::point::run_point;
use serde_json::{Value, to_value};
use std::collections::HashMap;
use serde::Serialize;

pub async fn run_case(context: &CaseContext<'_>) -> CaseResult {
    let mut point_vec: Vec<PointContext> = context.create_point();
    let mut point_value_register = Vec::<(String, PointResult)>::new();

    for mut point in point_vec.iter() {
        let result = run_point(&point).await;

        if !result.is_ok(){
            register_value(&mut point_value_register, String::from(point.get_id()), result);
            return Err(());
        } else {
            register_value(&mut point_value_register, String::from(point.get_id()), result);
        }



    }

    return Ok(point_value_register);
}


pub fn register_value(point_value_register: &mut Vec<(String, PointResult)>, name: String, result: PointResult) {
    point_value_register.push((name, result));
}