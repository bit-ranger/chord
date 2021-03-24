use chrono::Utc;
use common::value::to_json;

use common::error::Error;
use common::value::Json;
use res::CaseAssessStruct;

use crate::flow::case::arg::{CaseArgStruct, RenderContext};
use crate::flow::point;
use crate::model::app::AppContext;
use common::case::{CaseState};
use common::point::{PointAssess, PointState};
pub mod res;
pub mod arg;
use log::{trace, debug, info, warn};

pub async fn run(app_context: &dyn AppContext, case_arg: &mut CaseArgStruct<'_,'_,'_>) -> CaseAssessStruct {
    trace!("case start {}", case_arg.id());
    let start = Utc::now();
    let mut render_context = case_arg.create_render_context();
    let mut point_assess_vec = Vec::<Box<dyn PointAssess>>::new();
    for point_id in case_arg.point_id_vec().iter() {
        let pt_arg = case_arg.create_point(point_id, app_context, &render_context);
        if pt_arg.is_none(){
            warn!("case Err {}", case_arg.id());
            return CaseAssessStruct::new(case_arg.id(), start, Utc::now(),
            CaseState::Err(Error::new("010", format!("invalid point {}", point_id).as_str())));
        }
        let point_arg = pt_arg.unwrap();
        let point_assess = point::run(app_context, &point_arg).await;

        point_assess_vec.push(Box::new(point_assess));
        let pt_assess = point_assess_vec.last().unwrap();

        match pt_assess.state(){
            PointState::Ok(json) => {
                register_dynamic(&mut render_context, point_id, json).await;
            },
            _ => {
                info!("case Fail {}", case_arg.id());
                return CaseAssessStruct::new(case_arg.id(), start, Utc::now(), CaseState::Fail(point_assess_vec));
            }
        }
    }

    debug!("case Ok {}", case_arg.id());
    return CaseAssessStruct::new(case_arg.id(), start, Utc::now(), CaseState::Ok(point_assess_vec));
}


pub async fn register_dynamic(render_context: &mut RenderContext, pt_id: &str, result: &Json) {
    if let Json::Object(data) = render_context.data_mut(){
        data["dyn"][pt_id] = to_json(result).unwrap();
    }
}



