use std::panic::AssertUnwindSafe;

use async_std::future::timeout;
use chrono::{DateTime, Utc};
use futures::FutureExt;
use log::{debug, info, trace};

use chord::action::{Action, RenderContextUpdate, RunArg, Scope};
use chord::step::StepState;
use chord::Error;
use res::StepAssessStruct;

use crate::flow::step::arg::RunArgStruct;
use crate::flow::step::res::StepThen;
use crate::model::app::FlowApp;
use chord::value::{json, to_string_pretty, Value};

pub mod arg;
pub mod res;

pub async fn run(
    _: &dyn FlowApp,
    arg: &RunArgStruct<'_, '_, '_>,
    action: &dyn Action,
) -> StepAssessStruct {
    trace!("step start {}", arg.id());
    let start = Utc::now();
    let future = AssertUnwindSafe(action.run(arg)).catch_unwind();
    let timeout_value = timeout(arg.timeout(), future).await;
    if let Err(_) = timeout_value {
        return assess_create(arg, start, Err(Error::new("timeout", "timeout")));
    }
    let unwind_value = timeout_value.unwrap();
    if let Err(_) = unwind_value {
        return assess_create(arg, start, Err(Error::new("unwind", "unwind")));
    }
    let action_value = unwind_value.unwrap();
    return assess_create(arg, start, action_value);
}

fn assess_create(
    arg: &RunArgStruct<'_, '_, '_>,
    start: DateTime<Utc>,
    action_value: Result<Box<dyn Scope>, Error>,
) -> StepAssessStruct {
    return match action_value {
        Ok(scope) => {
            if let Some(condition) = arg.assert() {
                if assert(arg, condition, Ok(scope.as_value())) {
                    debug!("step Ok   {}", arg.id());
                    let then = choose_then(arg, Ok(scope.as_value()));
                    StepAssessStruct::new(
                        arg.id().clone(),
                        start,
                        Utc::now(),
                        StepState::Ok(scope),
                        then,
                    )
                } else {
                    info!(
                        "step Fail {}\n{}\n<<<\n{}",
                        arg.id(),
                        to_string_pretty(scope.as_value()).unwrap_or("".to_string()),
                        to_string_pretty(&arg.args(None).unwrap_or(Value::Null))
                            .unwrap_or("".to_string()),
                    );
                    StepAssessStruct::new(
                        arg.id().clone(),
                        start,
                        Utc::now(),
                        StepState::Fail(scope),
                        None,
                    )
                }
            } else {
                debug!("step Ok   {}", arg.id());
                let then = choose_then(arg, Ok(scope.as_value()));
                StepAssessStruct::new(
                    arg.id().clone(),
                    start,
                    Utc::now(),
                    StepState::Ok(scope),
                    then,
                )
            }
        }
        Err(e) => {
            let error = json!({
                "code": e.code(),
                "message": e.message()
            });
            if arg.catch_err() {
                trace!("step catch {}", arg.id());
                if let Some(condition) = arg.assert() {
                    if assert(arg, condition, Err(&error)) {
                        debug!("step Ok   {}", arg.id());
                        let then = choose_then(arg, Err(&error));
                        StepAssessStruct::new(
                            arg.id().clone(),
                            start,
                            Utc::now(),
                            StepState::Ok(Box::new(error)),
                            then,
                        )
                    } else {
                        info!(
                            "step Fail {}\n{}\n<<<\n{}",
                            arg.id(),
                            to_string_pretty(&error).unwrap_or("".to_string()),
                            to_string_pretty(&arg.args(None).unwrap_or(Value::Null))
                                .unwrap_or("".to_string()),
                        );
                        StepAssessStruct::new(
                            arg.id().clone(),
                            start,
                            Utc::now(),
                            StepState::Fail(Box::new(error)),
                            None,
                        )
                    }
                } else {
                    debug!("step Ok   {}", arg.id());
                    let then = choose_then(arg, Err(&error));
                    StepAssessStruct::new(
                        arg.id().clone(),
                        start,
                        Utc::now(),
                        StepState::Ok(Box::new(error)),
                        then,
                    )
                }
            } else {
                info!(
                    "step Err  {}\n{}\n<<<\n{}",
                    arg.id(),
                    to_string_pretty(&error).unwrap_or("".to_string()),
                    to_string_pretty(&arg.args(None).unwrap_or(Value::Null))
                        .unwrap_or("".to_string()),
                );
                StepAssessStruct::new(arg.id().clone(), start, Utc::now(), StepState::Err(e), None)
            }
        }
    };
}

fn choose_then(arg: &RunArgStruct<'_, '_, '_>, value: Result<&Value, &Value>) -> Option<StepThen> {
    let then_vec = arg.then();
    if then_vec.is_none() {
        return None;
    }
    for then in then_vec.unwrap() {
        let cond = then["if"].as_str();
        if cond.is_none() || assert(arg, cond.unwrap(), value) {
            let goto = then["goto"].as_str();
            let goto = if goto.is_none() {
                None
            } else {
                let result = arg.render_str(goto.unwrap());
                if result.is_err() {
                    None
                } else {
                    Some(result.unwrap())
                }
            };

            let reg = &then["reg"];
            let reg = if reg.is_none() {
                None
            } else {
                let result = arg.render_value(reg.unwrap(), None);
                if result.is_err() {
                    None
                } else {
                    Some(result.unwrap())
                }
            };
            return Some(StepThen::new(reg, goto));
        }
    }
    return None;
}

fn assert(arg: &RunArgStruct<'_, '_, '_>, condition: &str, value: Result<&Value, &Value>) -> bool {
    let assert_tpl = format!(
        "{{{{#if {condition}}}}}true{{{{else}}}}false{{{{/if}}}}",
        condition = condition
    );

    let assert_result = arg
        .render_str(
            assert_tpl.as_str(),
            match value {
                Ok(value) => Some(Box::new(AssertContext {
                    state: "Ok".to_string(),
                    name: "value".to_string(),
                    value: value.clone(),
                })),
                Err(value) => Some(Box::new(AssertContext {
                    state: "Err".to_string(),
                    name: "error".to_string(),
                    value: value.clone(),
                })),
            },
        )
        .unwrap_or("false".to_string());

    assert_result == "true"
}

struct AssertContext {
    state: String,
    name: String,
    value: Value,
}

impl RenderContextUpdate for AssertContext {
    fn update(&self, value: &mut Value) {
        if let Value::Object(map) = value {
            map.insert("state".to_string(), Value::String(self.state.clone()));
            map.insert(self.name.clone(), self.value.clone());
        }
    }
}
