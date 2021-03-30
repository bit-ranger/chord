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

pub async fn run(app_ctx: &dyn AppContext, arg: &CaseArgStruct<'_,'_,'_>) -> CaseAssessStruct {
    trace!("case start {}", arg.id());
    let start = Utc::now();
    let mut render_context = arg.create_render_context();
    let mut point_assess_vec = Vec::<Box<dyn PointAssess>>::new();
    for point_id in arg.point_id_vec().iter() {
        let point_arg = arg.create_point_arg(point_id, app_ctx, &render_context);
        if point_arg.is_none(){
            warn!("case Err {}", arg.id());
            return CaseAssessStruct::new(arg.id(), start, Utc::now(),
                                         CaseState::Err(Error::new("010", format!("invalid point {}", point_id).as_str())));
        }
        let point_arg = point_arg.unwrap();
        let point_assess = point::run(app_ctx, &point_arg).await;

        point_assess_vec.push(Box::new(point_assess));
        let point_assess = point_assess_vec.last().unwrap();

        match point_assess.state(){
            PointState::Ok(json) => {
                register_dynamic(&mut render_context, point_id, json).await;
            },
            _ => {
                info!("case Fail {}", arg.id());
                return CaseAssessStruct::new(arg.id(), start, Utc::now(), CaseState::Fail(point_assess_vec));
            }
        }
    }

    debug!("case Ok {}", arg.id());
    return CaseAssessStruct::new(arg.id(), start, Utc::now(), CaseState::Ok(point_assess_vec));
}


pub async fn register_dynamic(render_context: &mut RenderContext, pt_id: &str, result: &Json) {
    if let Json::Object(data) = render_context.data_mut(){
        data["dyn"][pt_id] = to_json(result).unwrap();
    }
}



