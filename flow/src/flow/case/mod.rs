use chrono::Utc;
use common::value::to_json;

use common::error::Error;
use common::value::Json;
use result::CaseAssessStruct;

use crate::flow::case::arg::{CaseArgStruct, RenderContext};
use crate::flow::point;
use crate::model::app::AppContext;
use common::case::{CaseAssess, CaseState};
use common::point::{PointAssess, PointState};
pub mod result;
pub mod arg;

pub async fn run(app_context: &dyn AppContext, case_arg: &mut CaseArgStruct<'_,'_,'_>) -> CaseAssessStruct {
    let start = Utc::now();
    let mut render_context = case_arg.create_render_context();
    let mut point_assess_vec = Vec::<dyn PointAssess>::new();
    for point_id in case_arg.point_id_vec().iter() {
        let point_arg = case_arg.create_point(point_id, app_context, &render_context);
        if point_arg.is_none(){
            return CaseAssessStruct::new(case_arg.id(), start, Utc::now(),
            CaseState::Err(Error::new("000", "invalid point")));
        }
        let point_arg = point_arg.unwrap();
        let point_assess = point::run(app_context, &point_arg).await;

        point_assess_vec.push(point_assess);
        let point_assess = point_assess_vec.last().unwrap();

        match point_assess.state(){
            PointState::Ok(json) => {
                register_dynamic(&mut render_context, point_id, json).await;
            },
            _ => {
                return CaseAssessStruct::new(case_arg.id(), start, Utc::now(), CaseState::PointFail(point_assess_vec));
            }
        }
    }

    return CaseAssessStruct::new(case_arg.id(), start, Utc::now(), CaseState::Ok(point_assess_vec));
}


pub async fn register_dynamic(render_context: &mut RenderContext, point_id: &str, result: &Json) {
    if let Json::Object(data) = render_context.data_mut(){
        data["dyn"][point_id] = to_json(result).unwrap();
    }
}



