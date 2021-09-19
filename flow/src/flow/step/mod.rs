use std::panic::AssertUnwindSafe;

use async_std::future::timeout;
use chrono::{DateTime, Utc};
use futures::FutureExt;
use log::{debug, info, trace};

use chord::action::{Action, RunArg, Scope};
use chord::step::StepState;
use chord::Error;
use res::StepAssessStruct;

use crate::flow::step::arg::RunArgStruct;
use crate::flow::step::res::StepThen;
use crate::model::app::FlowApp;
use chord::err;
use chord::value::{json, to_string_pretty, Value};

pub mod arg;
pub mod res;

pub async fn run(
    _: &dyn FlowApp,
    arg: &mut RunArgStruct<'_, '_, '_>,
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
    arg: &mut RunArgStruct<'_, '_, '_>,
    start: DateTime<Utc>,
    action_value: Result<Box<dyn Scope>, Error>,
) -> StepAssessStruct {
    if let Err(e) = action_value.as_ref() {
        if !arg.catch_err() {
            let args = arg.args(None).unwrap_or(Value::Null);
            info!(
                "step Err  {}\n{}\n<<<\n{}",
                arg.id(),
                to_string_pretty(&to_value(&e)).unwrap_or("".to_string()),
                to_string_pretty(&args).unwrap_or("".to_string()),
            );
            return StepAssessStruct::new(
                arg.id().clone(),
                start,
                Utc::now(),
                StepState::Err(e.clone()),
                None,
            );
        } else {
            if let Value::Object(map) = arg.render_context() {
                map.insert("state".to_string(), Value::String("Err".to_string()));
                map.insert("value".to_string(), to_value(&e));
            }
        }
    } else if let Ok(sc) = action_value.as_ref() {
        if let Value::Object(map) = arg.render_context() {
            map.insert("state".to_string(), Value::String("Ok".to_string()));
            map.insert("value".to_string(), sc.value().clone());
        }
    }

    let then = assert_and_then(arg);
    // let value = arg.render_context()["value"].clone();
    return if let Err(e) = then {
        let args = arg.args(None).unwrap_or(Value::Null);
        info!(
            "step Err  {}\n{}\n<<<\n{}",
            arg.id(),
            to_string_pretty(&to_value(&e)).unwrap_or("".to_string()),
            to_string_pretty(&args).unwrap_or("".to_string()),
        );
        StepAssessStruct::new(arg.id().clone(), start, Utc::now(), StepState::Err(e), None)
    } else {
        let scope = action_value.unwrap();
        let (ar, at) = then.unwrap();
        if ar {
            debug!("step Ok   {}", arg.id());
            StepAssessStruct::new(
                arg.id().clone(),
                start,
                Utc::now(),
                StepState::Ok(scope),
                at,
            )
        } else {
            info!(
                "step Fail {}\n{}\n<<<\n{}",
                arg.id(),
                to_string_pretty(scope.value()).unwrap_or("".to_string()),
                to_string_pretty(scope.args()).unwrap_or("".to_string()),
            );
            StepAssessStruct::new(
                arg.id().clone(),
                start,
                Utc::now(),
                StepState::Fail(scope),
                None,
            )
        }
    };
}

fn assert_and_then(arg: &RunArgStruct<'_, '_, '_>) -> Result<(bool, Option<StepThen>), Error> {
    let assert_success = value_assert(arg, arg.assert())?;
    return if !assert_success {
        Ok((false, None))
    } else {
        Ok((true, choose_then(arg)?))
    };
}

fn value_assert(arg: &RunArgStruct<'_, '_, '_>, condition: Option<&str>) -> Result<bool, Error> {
    if let Some(condition) = condition {
        assert(arg, condition)
    } else {
        Ok(true)
    }
}

fn choose_then(arg: &RunArgStruct<'_, '_, '_>) -> Result<Option<StepThen>, Error> {
    let then_vec = arg.then();
    if then_vec.is_none() {
        return Ok(None);
    }
    for then in then_vec.unwrap() {
        let cond: Option<&Value> = then.get("if");
        if cond.is_none()
            || cond.unwrap().as_str().is_none()
            || value_assert(arg, cond.unwrap().as_str())?
        {
            let goto = then.get("goto");
            let goto = if goto.is_none() {
                None
            } else if let Value::String(goto) = goto.unwrap() {
                Some(arg.render_str(goto.as_str(), None)?)
            } else {
                None
            };

            let reg = then.get("reg");
            let reg = if reg.is_none() {
                None
            } else if let Value::Object(_) = reg.unwrap() {
                let value = arg.render_value(reg.unwrap(), None)?;
                Some(
                    value
                        .as_object()
                        .map(|m| m.clone())
                        .ok_or(err!("001", "invalid reg"))?,
                )
            } else {
                None
            };
            return Ok(Some(StepThen::new(reg, goto)));
        }
    }
    return Ok(None);
}

fn assert(arg: &RunArgStruct<'_, '_, '_>, condition: &str) -> Result<bool, Error> {
    let assert_tpl = format!(
        "{{{{#if {condition}}}}}true{{{{else}}}}false{{{{/if}}}}",
        condition = condition
    );
    let assert_result = arg.render_str(assert_tpl.as_str(), None)?;
    Ok(assert_result == "true")
}

fn to_value(e: &Error) -> Value {
    json!({
        "code": e.code(),
        "message": e.message()
    })
}
