use std::panic::AssertUnwindSafe;

use async_std::future::timeout;
use chrono::{DateTime, Utc};
use futures::FutureExt;
use log::{debug, info, trace};

use chord::action::{Action, Context, RunArg, Scope};
use chord::step::StepState;
use chord::Error;
use res::StepAssessStruct;

use crate::flow::step::arg::RunArgStruct;
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
        return assert_assess(arg, start, Err(Error::new("timeout", "timeout")));
    }
    let unwind_value = timeout_value.unwrap();
    if let Err(_) = unwind_value {
        return assert_assess(arg, start, Err(Error::new("unwind", "unwind")));
    }
    let action_value = unwind_value.unwrap();
    return assert_assess(arg, start, action_value);
}

fn assert_assess(
    arg: &RunArgStruct<'_, '_, '_>,
    start: DateTime<Utc>,
    action_value: Result<Box<dyn Scope>, Error>,
) -> StepAssessStruct {
    return match action_value {
        Ok(scope) => {
            if let Some(condition) = arg.assert() {
                if assert(arg, condition, Ok(scope.as_value())) {
                    debug!("step Ok   {}", arg.id());
                    StepAssessStruct::new(arg.id().clone(), start, Utc::now(), StepState::Ok(scope))
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
                    )
                }
            } else {
                debug!("step Ok   {}", arg.id());
                StepAssessStruct::new(arg.id().clone(), start, Utc::now(), StepState::Ok(scope))
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
                        StepAssessStruct::new(
                            arg.id().clone(),
                            start,
                            Utc::now(),
                            StepState::Ok(Box::new(error)),
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
                        )
                    }
                } else {
                    debug!("step Ok   {}", arg.id());
                    StepAssessStruct::new(
                        arg.id().clone(),
                        start,
                        Utc::now(),
                        StepState::Ok(Box::new(error)),
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
                StepAssessStruct::new(arg.id().clone(), start, Utc::now(), StepState::Err(e))
            }
        }
    };
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

impl Context for AssertContext {
    fn update(&self, value: &mut Value) {
        if let Value::Object(map) = value {
            map.insert("state".to_string(), Value::String(self.state.clone()));
            map.insert(self.name.clone(), self.value.clone());
        }
    }
}
