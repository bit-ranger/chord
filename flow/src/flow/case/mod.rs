use chord_common::value::Json;
use chrono::Utc;
use res::CaseAssessStruct;

use crate::flow::case::arg::{CaseArgStruct, RenderContext};
use crate::flow::step;
use crate::model::app::FlowContext;
use chord_common::case::CaseState;
use chord_common::step::{RunArg, StepAssess, StepState};
pub mod arg;
pub mod res;
use crate::flow::step::arg::{assert, render};
use crate::flow::step::res::StepAssessStruct;
use chord_common::err;
use log::{debug, info, trace, warn};

pub async fn run(flow_ctx: &dyn FlowContext, arg: CaseArgStruct) -> CaseAssessStruct {
    trace!("case start {}", arg.id());
    let start = Utc::now();
    let mut render_context = arg.create_render_context();
    let mut step_assess_vec = Vec::<Box<dyn StepAssess>>::new();
    for (step_id, step_runner) in arg.step_runner_vec() {
        let step_arg = arg.step_arg_create(step_id, flow_ctx, &render_context);
        if step_arg.is_none() {
            warn!("case  Err {}", arg.id());
            return CaseAssessStruct::new(
                arg.id().clone(),
                start,
                Utc::now(),
                CaseState::Err(err!("010", format!("invalid step {}", step_id))),
            );
        }
        let step_arg = step_arg.unwrap();
        let step_assess = step::run(flow_ctx, &step_arg, step_runner.as_ref()).await;

        let config_raw = step_arg.config().to_string();
        match step_assess {
            StepAssessStruct {
                state: StepState::Err(e),
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
                info!("step Err  {} - {} <<< {}", id, e, config_rendered);
                info!("case  Fail {}", arg.id());
                return CaseAssessStruct::new(
                    arg.id().clone(),
                    start,
                    Utc::now(),
                    CaseState::Fail(step_assess_vec),
                );
            }
            StepAssessStruct {
                state: StepState::Fail(json),
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
                info!("step Fail {} - {} <<< {}", arg.id(), json, config_rendered);
                let step_assess = StepAssessStruct::new(id, start, end, StepState::Fail(json));
                step_assess_vec.push(Box::new(step_assess));
                info!("case  Fail {}", arg.id());
                return CaseAssessStruct::new(
                    arg.id().clone(),
                    start,
                    Utc::now(),
                    CaseState::Fail(step_assess_vec),
                );
            }
            StepAssessStruct {
                state: StepState::Ok(json),
                id,
                start,
                end,
            } => {
                let assert_present = step_arg.meta_str(vec!["assert"]).await;
                register_dynamic(&mut render_context, step_id, &json).await;
                if let Some(con) = assert_present {
                    if assert(flow_ctx.get_handlebars(), &mut render_context, &con).await {
                        debug!("step Ok   {}", id);
                        let step_assess =
                            StepAssessStruct::new(id, start, end, StepState::Ok(json));
                        step_assess_vec.push(Box::new(step_assess));
                    } else {
                        let config_rendered = render(
                            flow_ctx.get_handlebars(),
                            &render_context,
                            config_raw.as_str(),
                        )
                        .unwrap_or("".to_owned());
                        info!("step Fail {} - {} <<< {}", id, json, config_rendered);
                        let step_assess =
                            StepAssessStruct::new(id, start, end, StepState::Fail(json));
                        step_assess_vec.push(Box::new(step_assess));
                        info!("case  Fail {}", arg.id());
                        return CaseAssessStruct::new(
                            arg.id().clone(),
                            start,
                            Utc::now(),
                            CaseState::Fail(step_assess_vec),
                        );
                    }
                } else {
                    debug!("step Ok   {}", id);
                    let step_assess = StepAssessStruct::new(id, start, end, StepState::Ok(json));
                    step_assess_vec.push(Box::new(step_assess));
                }
            }
        }
    }

    debug!("case Ok {}", arg.id());
    return CaseAssessStruct::new(
        arg.id().clone(),
        start,
        Utc::now(),
        CaseState::Ok(step_assess_vec),
    );
}

pub async fn register_dynamic(render_context: &mut RenderContext, sid: &str, value: &Json) {
    if let Json::Object(data) = render_context.data_mut() {
        data["step"][sid]["value"] = value.clone();
        data["curr"]["value"] = value.clone();
    }
}
