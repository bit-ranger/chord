use chrono::Utc;
use common::value::to_json;

use common::error::Error;
use common::value::Json;
use result::CaseAssessStruct;

use crate::flow::case::arg::{CaseArgStruct, RenderContext};
use crate::flow::point;
use crate::model::app::AppContext;
use common::case::{CaseState};
use common::point::{PointAssess, PointState};
pub mod result;
pub mod arg;

pub async fn run(app_context: &dyn AppContext, case_arg: &mut CaseArgStruct<'_,'_,'_>) -> CaseAssessStruct {
    let start = Utc::now();
    let mut render_context = case_arg.create_render_context();
    let mut pt_assess_vec = Vec::<Box<dyn PointAssess>>::new();
    for pt_id in case_arg.pt_id_vec().iter() {
        let pt_arg = case_arg.create_point(pt_id, app_context, &render_context);
        if pt_arg.is_none(){
            return CaseAssessStruct::new(case_arg.id(), start, Utc::now(),
            CaseState::Err(Error::new("000", "invalid point")));
        }
        let pt_arg = pt_arg.unwrap();
        let pt_assess = point::run(app_context, &pt_arg).await;

        pt_assess_vec.push(Box::new(pt_assess));
        let pt_assess = pt_assess_vec.last().unwrap();

        match pt_assess.state(){
            PointState::Ok(json) => {
                register_dynamic(&mut render_context, pt_id, json).await;
            },
            _ => {
                return CaseAssessStruct::new(case_arg.id(), start, Utc::now(), CaseState::Fail(pt_assess_vec));
            }
        }
    }

    return CaseAssessStruct::new(case_arg.id(), start, Utc::now(), CaseState::Ok(pt_assess_vec));
}


pub async fn register_dynamic(render_context: &mut RenderContext, pt_id: &str, result: &Json) {
    if let Json::Object(data) = render_context.data_mut(){
        data["dyn"][pt_id] = to_json(result).unwrap();
    }
}



