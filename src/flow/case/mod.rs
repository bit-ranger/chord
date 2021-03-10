use crate::flow::case::model::{CaseContextStruct, RenderContext};
use crate::flow::point::{run_point, assert};
use crate::model::context::AppContext;
use crate::model::context::{CaseResult, PointResult};
use crate::model::error::Error;
use crate::model::value::Json;
use serde_json::to_value;

pub mod model;

pub async fn run_case(app_context: &dyn AppContext, context: &mut CaseContextStruct<'_,'_>) -> CaseResult {
    let mut render_context = context.create_render_context();
    let mut point_result_vec = Vec::<(String, PointResult)>::new();
    for point_id in context.get_point_id_vec().iter() {
        let point = context.create_point(point_id, app_context, &render_context);
        if point.is_none(){
            return Err(Error::new("000", format!("invalid point {}", point_id).as_str()));
        }
        let point = point.unwrap();
        let result = run_point(&point).await;

        match &result {
            Ok(r) => {
                let assert_true = assert(&point, r).await;
                if assert_true {
                    register_dynamic(&mut render_context, point_id, r).await;
                    point_result_vec.push((String::from(point_id), result));
                } else {
                    point_result_vec.push((String::from(point_id),
                                           Err(Error::new("000", "assert failure"))));
                    return Err(Error::new("002", format!("point assert failure {}", point_id).as_str()));
                }
            },
            Err(_) =>  {
                point_result_vec.push((String::from(point_id), result));
                return Err(Error::new("001", format!("point run failure {}", point_id).as_str()));
            }
        }

    }

    return Ok(point_result_vec);
}


pub async fn register_dynamic(render_context: &mut RenderContext, point_id: &str, result: &Json) {
    if let Json::Object(data) = render_context.data_mut(){
        data["dyn"][point_id] = to_value(result).unwrap();
    }
}



