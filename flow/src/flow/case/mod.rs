use chord_common::value::Json;
use chrono::Utc;
use res::CaseAssessStruct;

use crate::flow::case::arg::{CaseArgStruct, RenderContext};
use crate::flow::point;
use crate::model::app::FlowContext;
use chord_common::case::CaseState;
use chord_common::point::{PointArg, PointAssess, PointState};
pub mod arg;
pub mod res;
use crate::flow::point::arg::{assert, render};
use crate::flow::point::res::PointAssessStruct;
use async_std::task_local;
use chord_common::err;
use log::{debug, info, trace, warn};
use std::cell::RefCell;

task_local! {
    pub static CASE_ID: RefCell<usize> = RefCell::new(0);
}

pub async fn run(flow_ctx: &dyn FlowContext, arg: &CaseArgStruct) -> CaseAssessStruct {
    trace!("case start {}", arg.id());
    let start = Utc::now();
    let mut render_context = arg.create_render_context();
    let mut point_assess_vec = Vec::<Box<dyn PointAssess>>::new();
    for (point_id, point_runner) in arg.point_runner_vec() {
        let point_arg = arg.create_point_arg(point_id, flow_ctx, &render_context);
        if point_arg.is_none() {
            warn!("case  Err {}", arg.id());
            return CaseAssessStruct::new(
                arg.id(),
                start,
                Utc::now(),
                CaseState::Err(err!("010", format!("invalid point {}", point_id))),
            );
        }
        let point_arg = point_arg.unwrap();
        let point_assess = point::run(flow_ctx, &point_arg, point_runner.as_ref()).await;

        let config_raw = point_arg.config().to_string();
        match point_assess {
            PointAssessStruct {
                state: PointState::Err(e),
                id,
                start,
                end: _,
            } => {
                let config_rendered = render(
                    flow_ctx.get_handlebars(),
                    &render_context,
                    config_raw.as_str(),
                )
                .unwrap_or("".to_owned());
                info!("point Err  {} - {} <<< {}", id, e, config_rendered);
                info!("case  Fail {}", arg.id());
                return CaseAssessStruct::new(
                    arg.id(),
                    start,
                    Utc::now(),
                    CaseState::Fail(point_assess_vec),
                );
            }
            PointAssessStruct {
                state: PointState::Fail(json),
                id,
                start,
                end,
            } => {
                let config_rendered = render(
                    flow_ctx.get_handlebars(),
                    &render_context,
                    config_raw.as_str(),
                )
                .unwrap_or("".to_owned());
                info!("point Fail {} - {} <<< {}", arg.id(), json, config_rendered);
                let point_assess =
                    PointAssessStruct::new(id.as_str(), start, end, PointState::Fail(json));
                point_assess_vec.push(Box::new(point_assess));
                info!("case  Fail {}", arg.id());
                return CaseAssessStruct::new(
                    arg.id(),
                    start,
                    Utc::now(),
                    CaseState::Fail(point_assess_vec),
                );
            }
            PointAssessStruct {
                state: PointState::Ok(json),
                id,
                start,
                end,
            } => {
                let assert_present = point_arg.meta_str(vec!["assert"]).await;
                register_dynamic(&mut render_context, point_id, &json).await;
                if let Some(con) = assert_present {
                    if assert(flow_ctx.get_handlebars(), &mut render_context, &con).await {
                        debug!("point Ok   {}", id);
                        let point_assess =
                            PointAssessStruct::new(id.as_str(), start, end, PointState::Ok(json));
                        point_assess_vec.push(Box::new(point_assess));
                    } else {
                        let config_rendered = render(
                            flow_ctx.get_handlebars(),
                            &render_context,
                            config_raw.as_str(),
                        )
                        .unwrap_or("".to_owned());
                        info!("point Fail {} - {} <<< {}", id, json, config_rendered);
                        let point_assess =
                            PointAssessStruct::new(id.as_str(), start, end, PointState::Fail(json));
                        point_assess_vec.push(Box::new(point_assess));
                        info!("case  Fail {}", arg.id());
                        return CaseAssessStruct::new(
                            arg.id(),
                            start,
                            Utc::now(),
                            CaseState::Fail(point_assess_vec),
                        );
                    }
                } else {
                    debug!("point Ok   {}", id);
                    let point_assess =
                        PointAssessStruct::new(id.as_str(), start, end, PointState::Ok(json));
                    point_assess_vec.push(Box::new(point_assess));
                }
            }
        }
    }

    debug!("case Ok {}", arg.id());
    return CaseAssessStruct::new(arg.id(), start, Utc::now(), CaseState::Ok(point_assess_vec));
}

pub async fn register_dynamic(render_context: &mut RenderContext, pt_id: &str, result: &Json) {
    if let Json::Object(data) = render_context.data_mut() {
        data["dyn"][pt_id] = result.clone();
        data["res"] = result.clone();
    }
}
