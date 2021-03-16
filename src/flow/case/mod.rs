use crate::flow::case::model::{CaseContextStruct, RenderContext};
use crate::flow::point::{run_point, assert};
use crate::model::context::{AppContext, CaseResultStruct, PointResultInner, CaseError};
use crate::model::context::{CaseResult};
use crate::model::value::Json;
use serde_json::to_value;
use chrono::{Utc};
use std::time::Duration;
use log::info;


pub mod model;

pub async fn run_case(app_context: &dyn AppContext, context: &mut CaseContextStruct<'_,'_>) -> CaseResult {
    let start = Utc::now();
    async_std::task::sleep(Duration::from_secs(5)).await;
    let mut render_context = context.create_render_context();
    let mut point_result_vec = Vec::<PointResultInner>::new();
    for point_id in context.get_point_id_vec().iter() {
        let point = context.create_point(point_id, app_context, &render_context);
        if point.is_none(){
            let result_struct = CaseResultStruct::new(point_result_vec, context.id(), start, Utc::now());
            return CaseResult::Err(CaseError::attach("000", "invalid point", result_struct));
        }
        let point = point.unwrap();
        let result = run_point(&point).await;
        point_result_vec.push( result);
        let result = point_result_vec.last().unwrap();

        match result {
            Ok(r) => {
                let assert_true = assert(&point, r.result()).await;
                if assert_true {
                    register_dynamic(&mut render_context, point_id, r.result()).await;
                } else {
                    let result_struct = CaseResultStruct::new(point_result_vec, context.id(), start, Utc::now());
                    return CaseResult::Err(CaseError::attach("020", "assert failure", result_struct));
                }
            },
            Err(_) =>  {
                let result_struct = CaseResultStruct::new(point_result_vec, context.id(), start, Utc::now());
                return CaseResult::Err(CaseError::attach("010", "point run failure", result_struct));
            }
        }

    }

    let result_struct = CaseResultStruct::new(point_result_vec, context.id(), start, Utc::now());
    return CaseResult::Ok(result_struct);

}


pub async fn register_dynamic(render_context: &mut RenderContext, point_id: &str, result: &Json) {
    if let Json::Object(data) = render_context.data_mut(){
        data["dyn"][point_id] = to_value(result).unwrap();
    }
}



