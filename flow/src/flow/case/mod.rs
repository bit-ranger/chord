use chrono::Utc;
use common::value::to_json;

use common::error::Error;
use common::value::Json;
use result::CaseAssessStruct;

use crate::flow::case::arg::{CaseArgStruct, RenderContext};
use crate::flow::point;
use crate::model::app::AppContext;
use common::case::{CaseResult, CaseState};
use common::point::{PointResult, PointState};

pub mod result;
pub mod arg;

pub async fn run(app_context: &dyn AppContext, case_arg: &mut CaseArgStruct<'_,'_>) -> CaseResult {
    let start = Utc::now();
    let mut render_context = case_arg.create_render_context();
    let mut point_result_vec = Vec::<(String, PointResult)>::new();
    for point_id in case_arg.point_id_vec().iter() {
        let point_arg = case_arg.create_point(point_id, app_context, &render_context);
        if point_arg.is_none(){
            return Err(Error::new("000", "invalid point"));
        }
        let point_context = point_arg.unwrap();
        let point_result = point::run(app_context, &point_context).await;

        // let (point_id, point_result) = point_result_vec.last().unwrap();
        match &point_result {
            Ok(r) => {
                match r.state(){
                    PointState::Ok => {
                        register_dynamic(&mut render_context, point_id, r.result()).await;
                        point_result_vec.push((point_id.clone(), point_result));
                    },
                    PointState::Error(e) => {
                        let state = CaseState::PointError(e.clone());
                        point_result_vec.push((point_id.clone(), point_result));
                        let result_struct = CaseAssessStruct::new(point_result_vec, case_arg.id(), start, Utc::now(), state);
                        return Ok(Box::new(result_struct));
                    },
                    PointState::Failure=> {
                        point_result_vec.push((point_id.clone(), point_result));
                        let state = CaseState::PointFailure;
                        let result_struct = CaseAssessStruct::new(point_result_vec, case_arg.id(), start, Utc::now(), state);
                        return Ok(Box::new(result_struct));
                    }
                }
            },
            Err(e) =>  {
                let state = CaseState::PointError(e.clone());
                let result_struct = CaseAssessStruct::new(point_result_vec, case_arg.id(), start, Utc::now(), state);
                return Ok(Box::new(result_struct));
            }
        }
    }

    let result_struct = CaseAssessStruct::new(point_result_vec, case_arg.id(), start, Utc::now(), CaseState::Ok);
    return Ok(Box::new(result_struct));
}


pub async fn register_dynamic(render_context: &mut RenderContext, point_id: &str, result: &Json) {
    if let Json::Object(data) = render_context.data_mut(){
        data["dyn"][point_id] = to_json(result).unwrap();
    }
}



