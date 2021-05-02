use chrono::Utc;
use chord_common::value::Json;
use res::CaseAssessStruct;

use crate::flow::case::arg::{CaseArgStruct, RenderContext};
use crate::flow::point;
use crate::model::app::AppContext;
use chord_common::case::{CaseState};
use chord_common::point::{PointAssess, PointState};
pub mod res;
pub mod arg;
use log::{trace, debug, info, warn};
use chord_common::err;
use crate::flow::point::arg::{assert};
use crate::flow::point::res::PointAssessStruct;

pub async fn run(app_ctx: &dyn AppContext, arg: &CaseArgStruct) -> CaseAssessStruct {
    trace!("case start {}", arg.id());
    let start = Utc::now();
    let mut render_context = arg.create_render_context();
    let mut point_assess_vec = Vec::<Box<dyn PointAssess>>::new();
    for point_id in arg.point_id_vec().iter() {
        let point_arg = arg.create_point_arg(point_id, app_ctx, &render_context);
        if point_arg.is_none(){
            warn!("case Err {}", arg.id());
            return CaseAssessStruct::new(arg.id(), start, Utc::now(),
                                         CaseState::Err(err!("010", format!("invalid point {}", point_id))));
        }
        let point_arg = point_arg.unwrap();
        let point_assess = point::run(app_ctx, &point_arg).await;

        match point_assess{
            PointAssessStruct {state: PointState::Ok(json), id,start, end} => {
                if let Some(con) =  point_arg.meta_str(vec!["assert"]).await{
                    register_dynamic(&mut render_context, point_id, &json).await;
                    if assert(app_ctx.get_handlebars(), &mut render_context, &con).await {
                        let point_assess = PointAssessStruct::new(id.as_str(), start, end,PointState::Ok(json));
                        point_assess_vec.push(Box::new(point_assess));
                    } else {
                        info!("point Fail {} - {} \n", arg.id(), json);
                        let point_assess = PointAssessStruct::new(id.as_str(), start, end, PointState::Fail(json));
                        point_assess_vec.push(Box::new(point_assess));
                        info!("case Fail {}", arg.id());
                        return CaseAssessStruct::new(arg.id(), start, Utc::now(), CaseState::Fail(point_assess_vec));
                    }
                }
            },
            _ => {
                point_assess_vec.push(Box::new(point_assess));
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
        data["dyn"][pt_id] = result.clone();
        data["res"] = result.clone();
    }
}


