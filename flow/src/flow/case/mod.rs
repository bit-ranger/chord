use async_std::sync::Arc;
use chrono::Utc;
use log::{debug, info, trace, warn};

use chord::action::RunArg;
use chord::case::{CaseAssess, CaseState};
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

pub struct CaseRunner {
    flow_ctx: Arc<dyn Context>,
    case_arg: CaseArgStruct,
}

impl CaseRunner {
    pub async fn new(flow_ctx: Arc<dyn Context>, case_arg: CaseArgStruct) -> CaseRunner {
        CaseRunner { flow_ctx, case_arg }
    }

    pub async fn run(self) -> Box<dyn CaseAssess> {
        Box::new(self.run0().await)
    }

    async fn run0(self) -> CaseAssessStruct {
        let flow_ctx = self.flow_ctx.as_ref();
        let case_arg = &self.case_arg;

        trace!("case start {}", case_arg.id());
        let start = Utc::now();
        let mut render_context = case_arg.create_render_context();
        let step_vec = case_arg.step_vec();
        let mut step_assess_vec = Vec::<Box<dyn StepAssess>>::new();
        let mut prev_step_idx = None;
        loop {
            curr_reset(&mut render_context).await;

            let step_idx = self.next_step_idx(prev_step_idx, &render_context).await;
            if let None = step_idx {
                debug!("case Ok {}", case_arg.id());
                return CaseAssessStruct::new(
                    case_arg.id().clone(),
                    start,
                    Utc::now(),
                    CaseState::Ok(TailDropVec::from(step_assess_vec)),
                    self.case_arg.take_data(),
                );
            };

            let (step_id, action) = step_vec.get(step_idx.unwrap()).unwrap();
            let step_id = step_id.as_str();
            let action = action.as_ref();
            prev_step_idx = step_idx;

            let step_arg = case_arg.step_arg_create(step_id, flow_ctx, &render_context);
            if step_arg.is_none() {
                warn!("case Err {}", case_arg.id());
                return CaseAssessStruct::new(
                    case_arg.id().clone(),
                    start,
                    Utc::now(),
                    CaseState::Err(err!("010", format!("invalid step {}", step_id))),
                    self.case_arg.take_data(),
                );
            }
            let step_arg = step_arg.unwrap();
            let step_assess = step::run(flow_ctx, &step_arg, action).await;

            let step_arg_id = step_arg.id().clone();
            let step_arg_args = step_arg
                .render_value(step_arg.args())
                .unwrap_or(Value::Null);
            let step_arg_assert = step_arg.assert().map(|s| s.to_owned());
            let step_arg_catch_err = step_arg.catch_err();

            curr_register(&mut render_context, step_assess.state()).await;
            step_register(
                &mut render_context,
                step_assess.id().step(),
                step_assess.state(),
            )
            .await;

            if step_assess.state.is_fail() {
                // never reach
                panic!("step state cannot be fail");
            } else if step_assess.state.is_ok() {
                let step_assess =
                    step_assess_assert(flow_ctx, &mut render_context, step_assess, step_arg_assert)
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
                    info!("case Fail {}", case_arg.id());
                    return CaseAssessStruct::new(
                        case_arg.id().clone(),
                        start,
                        Utc::now(),
                        CaseState::Fail(TailDropVec::from(step_assess_vec)),
                        self.case_arg.take_data(),
                    );
                }
            } else if step_assess.state.is_err() {
                if !step_arg_catch_err {
                    debug!("step Err  {}", step_arg_id);
                    step_assess_vec.push(Box::new(step_assess));
                    info!("case Fail {}", case_arg.id());
                    return CaseAssessStruct::new(
                        case_arg.id().clone(),
                        start,
                        Utc::now(),
                        CaseState::Fail(TailDropVec::from(step_assess_vec)),
                        self.case_arg.take_data(),
                    );
                } else {
                    trace!("step catch {}", step_arg_id);
                    let step_assess = step_assess_assert(
                        flow_ctx,
                        &mut render_context,
                        step_assess,
                        step_arg_assert,
                    )
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
                        info!("case Fail {}", case_arg.id());
                        return CaseAssessStruct::new(
                            case_arg.id().clone(),
                            start,
                            Utc::now(),
                            CaseState::Fail(TailDropVec::from(step_assess_vec)),
                            self.case_arg.take_data(),
                        );
                    }
                }
            }
        }
    }

    async fn next_step_idx(
        &self,
        step_idx: Option<usize>,
        render_context: &RenderContext,
    ) -> Option<usize> {
        if let None = step_idx {
            return Some(0);
        }

        let step_idx = step_idx.unwrap();
        let step_vec = self.case_arg.step_vec();
        let step_node = step_vec.get(step_idx);
        if let None = step_node {
            return None;
        }
        let (step_id, _) = step_node.unwrap();
        let step_arg =
            self.case_arg
                .step_arg_create(step_id, self.flow_ctx.as_ref(), render_context);
        if let None = step_arg {
            return None;
        }
        let step_arg = step_arg.unwrap();
        if let Some((step_id, cond)) = step_arg.goto() {
            if assert(self.flow_ctx.get_handlebars(), &render_context, cond).await {
                for (idx, (sid, _)) in step_vec.iter().enumerate() {
                    if sid == step_id {
                        return Some(idx);
                    }
                }
            }
        }

        return if step_idx == step_vec.len() - 1 {
            None
        } else {
            return Some(step_idx + 1);
        };
    }
}

/// step_assess.state cannot be Fail
async fn step_assess_assert(
    flow_ctx: &dyn Context,
    render_context: &RenderContext,
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

async fn step_register(render_context: &mut RenderContext, sid: &str, state: &StepState) {
    match state {
        StepState::Ok(scope) => {
            if let Value::Object(reg) = render_context.data_mut() {
                reg["step"][sid]["state"] = Value::String("Ok".to_owned());
                reg["step"][sid]["value"] = scope.as_value().clone();
            }
        }
        StepState::Err(e) => {
            if let Value::Object(reg) = render_context.data_mut() {
                reg["step"][sid]["state"] = Value::String("Err".to_owned());
                reg["step"][sid]["error"] = json!({
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

#[allow(dead_code)]
async fn step_unregister(render_context: &mut RenderContext, sid: &str) {
    if let Value::Object(reg) = render_context.data_mut() {
        if let Some(step) = reg["step"].as_object_mut() {
            step.remove(sid);
        }
    }
}

async fn curr_register(render_context: &mut RenderContext, state: &StepState) {
    match state {
        StepState::Ok(scope) => {
            if let Value::Object(reg) = render_context.data_mut() {
                reg["curr"]["state"] = Value::String("Ok".to_owned());
                reg["curr"]["value"] = scope.as_value().clone();
            }
        }
        StepState::Err(e) => {
            if let Value::Object(reg) = render_context.data_mut() {
                reg["curr"]["state"] = Value::String("Err".to_owned());
                reg["curr"]["error"] = json!({
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

async fn curr_reset(render_context: &mut RenderContext) {
    if let Value::Object(reg) = render_context.data_mut() {
        reg["curr"] = Value::Null;
    }
}
