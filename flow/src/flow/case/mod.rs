use chrono::Utc;
use log::{debug, info, trace, warn};

use chord::action::{RunArg, RunId};
use chord::case::CaseState;
use chord::collection::TailDropVec;
use chord::err;
use chord::step::{StepAssess, StepState};
use chord::value::{json, Value};
use res::CaseAssessStruct;

use crate::flow::case::arg::CaseArgStruct;
use crate::flow::step::res::StepAssessStruct;
use crate::flow::{assert, step};
use crate::model::app::{Context, RenderContext};

pub mod arg;
pub mod res;

pub async fn run(flow_ctx: &dyn Context, arg: CaseArgStruct) -> CaseAssessStruct {
    trace!("case start {}", arg.id());
    let start = Utc::now();
    let mut render_context = arg.create_render_context();
    let mut step_assess_vec = Vec::<Box<dyn StepAssess>>::new();
    for (step_id, action) in arg.step_vec().iter() {
        let step_arg = arg.step_arg_create(step_id, flow_ctx, &render_context);
        if step_arg.is_none() {
            warn!("case Err {}", arg.id());
            return CaseAssessStruct::new(
                arg.id().clone(),
                start,
                Utc::now(),
                arg.take_data(),
                CaseState::Err(err!("010", format!("invalid step {}", step_id))),
            );
        }
        let step_arg = step_arg.unwrap();
        let step_assess = step::run(flow_ctx, &step_arg, action.as_ref()).await;

        if step_assess.state.is_fail() {
            // never reach
            panic!("step state cannot be fail");
        } else if step_assess.state.is_ok() {
            let step_arg_id = step_arg.id().clone();
            let step_arg_args = step_arg
                .render_value(step_arg.args())
                .unwrap_or(Value::Null);

            let assert_present = step_arg.assert().map(|s| s.to_owned());
            let step_assess =
                step_assess_assert(flow_ctx, &mut render_context, step_assess, assert_present)
                    .await;

            if step_assess.state.is_ok() {
                debug!("step Ok   {}", step_arg_id);
                step_assess_vec.push(Box::new(step_assess));
            } else {
                if let StepState::Fail(scope) = &step_assess.state {
                    info!(
                        "step Fail {} - {} <<< {}",
                        step_arg_id,
                        scope.as_value(),
                        step_arg_args
                    );
                }

                step_assess_vec.push(Box::new(step_assess));
                info!("case Fail {}", arg.id());
                return CaseAssessStruct::new(
                    arg.id().clone(),
                    start,
                    Utc::now(),
                    arg.take_data(),
                    CaseState::Fail(TailDropVec::from(step_assess_vec)),
                );
            }
        } else if step_assess.state.is_err() {
            if !step_arg.catch_err() {
                debug!("step Err  {}", step_arg.id());
                step_assess_vec.push(Box::new(step_assess));
                info!("case Fail {}", arg.id());
                return CaseAssessStruct::new(
                    arg.id().clone(),
                    start,
                    Utc::now(),
                    arg.take_data(),
                    CaseState::Fail(TailDropVec::from(step_assess_vec)),
                );
            } else {
                trace!("step catch {}", step_arg.id());

                let step_arg_id = step_arg.id().clone();
                let step_arg_args = step_arg
                    .render_value(step_arg.args())
                    .unwrap_or(Value::Null);

                let assert_present = step_arg.assert().map(|s| s.to_owned());
                let step_assess =
                    step_assess_assert(flow_ctx, &mut render_context, step_assess, assert_present)
                        .await;
                if step_assess.state.is_ok() {
                    debug!("step Ok   {}", step_arg_id);
                    step_assess_vec.push(Box::new(step_assess));
                } else {
                    if let StepState::Fail(scope) = &step_assess.state {
                        info!(
                            "step Fail {} - {} <<< {}",
                            step_arg_id,
                            scope.as_value(),
                            step_arg_args
                        );
                    }

                    step_assess_vec.push(Box::new(step_assess));
                    info!("case Fail {}", arg.id());
                    return CaseAssessStruct::new(
                        arg.id().clone(),
                        start,
                        Utc::now(),
                        arg.take_data(),
                        CaseState::Fail(TailDropVec::from(step_assess_vec)),
                    );
                }
            }
        }
    }

    debug!("case Ok {}", arg.id());
    return CaseAssessStruct::new(
        arg.id().clone(),
        start,
        Utc::now(),
        arg.take_data(),
        CaseState::Ok(TailDropVec::from(step_assess_vec)),
    );
}

/// step_assess.state cannot be Fail
async fn step_assess_assert(
    flow_ctx: &dyn Context,
    render_context: &mut RenderContext,
    step_assess: StepAssessStruct,
    assert_present: Option<String>,
) -> StepAssessStruct {
    if step_assess.state().is_fail() {
        // never reach
        panic!("step state cannot be fail")
    }

    let StepAssessStruct {
        id,
        start,
        end,
        state,
    } = step_assess;
    step_register(render_context, id.step(), &state).await;
    if let Some(con) = assert_present {
        if assert(flow_ctx.get_handlebars(), &render_context, con.as_str()).await {
            match state {
                StepState::Ok(scope) => StepAssessStruct::new(id, start, end, StepState::Ok(scope)),
                StepState::Err(e) => StepAssessStruct::new(
                    id,
                    start,
                    end,
                    StepState::Ok(Box::new(json!({
                        "code": e.code(),
                        "message": e.message()
                    }))),
                ),
                StepState::Fail(_) => {
                    // never reach
                    panic!("step state cannot be fail")
                }
            }
        } else {
            match state {
                StepState::Ok(scope) => {
                    StepAssessStruct::new(id, start, end, StepState::Fail(scope))
                }
                StepState::Err(e) => StepAssessStruct::new(
                    id,
                    start,
                    end,
                    StepState::Fail(Box::new(json!({
                        "code": e.code(),
                        "message": e.message()
                    }))),
                ),
                StepState::Fail(_) => {
                    // never reach
                    panic!("step state cannot be fail")
                }
            }
        }
    } else {
        match state {
            StepState::Ok(scope) => StepAssessStruct::new(id, start, end, StepState::Ok(scope)),
            StepState::Err(e) => StepAssessStruct::new(
                id,
                start,
                end,
                StepState::Ok(Box::new(json!({
                    "code": e.code(),
                    "message": e.message()
                }))),
            ),
            StepState::Fail(_) => {
                // never reach
                panic!("step state cannot be fail")
            }
        }
    }
}

pub async fn step_register(render_context: &mut RenderContext, sid: &str, state: &StepState) {
    match state {
        StepState::Ok(scope) => {
            if let Value::Object(data) = render_context.data_mut() {
                data["step"][sid]["state"] = Value::String("Ok".to_owned());
                data["step"][sid]["value"] = scope.as_value().clone();

                data["curr"]["state"] = Value::String("Ok".to_owned());
                data["curr"]["value"] = scope.as_value().clone();
            }
        }
        StepState::Err(e) => {
            if let Value::Object(data) = render_context.data_mut() {
                data["step"][sid]["state"] = Value::String("Err".to_owned());
                data["step"][sid]["error"] = json!({
                    "code": e.code(),
                    "message": e.message()
                });

                data["curr"]["state"] = Value::String("Err".to_owned());
                data["curr"]["error"] = json!({
                    "code": e.code(),
                    "message": e.message()
                });
            }
        }
        StepState::Fail(_) => {
            // never reach
            panic!("step state cannot be fail");
        }
    }
}
