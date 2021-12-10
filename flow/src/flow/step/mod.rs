use std::panic::AssertUnwindSafe;

use async_std::future::timeout;
use chrono::{DateTime, Utc};
use futures::FutureExt;
use handlebars::TemplateRenderError;
use log::{debug, error, info, trace, warn};

use chord_core::action::{Action, Scope};
use chord_core::step::StepState;
use chord_core::value::json;
use chord_core::value::{to_string_pretty, Value};
use res::StepAssessStruct;
use Error::*;

use crate::flow::step::arg::RunArgStruct;
use crate::flow::step::res::StepThen;
use crate::flow::step::Error::ValueUnexpected;
use crate::model::app::FlowApp;

pub mod arg;
pub mod res;

#[derive(thiserror::Error, Debug)]
enum Error {
    #[error("timeout")]
    Timeout,

    #[error("unwind")]
    Unwind,

    #[error("`{0}` unexpect value `{1}`")]
    ValueUnexpected(String, String),

    #[error("`{0}` render error:\n{1}")]
    Render(String, TemplateRenderError),
}

pub async fn run(
    _: &dyn FlowApp,
    arg: &mut RunArgStruct<'_, '_, '_>,
    action: &dyn Action,
) -> StepAssessStruct {
    trace!("step start {}", arg.id());
    let start = Utc::now();
    let explain = action.explain(arg).await.unwrap_or(Value::Null);
    let future = AssertUnwindSafe(action.run(arg)).catch_unwind();
    let timeout_value = timeout(arg.timeout(), future).await;
    if let Err(_) = timeout_value {
        warn!("step timeout {}", arg.id());
        return assess_create(arg, start, explain, Err(Box::new(Timeout)));
    }
    let unwind_value = timeout_value.unwrap();
    if let Err(_) = unwind_value {
        error!("step unwind {}", arg.id());
        return assess_create(arg, start, explain, Err(Box::new(Unwind)));
    }
    let action_value = unwind_value.unwrap();
    return assess_create(arg, start, explain, action_value);
}

fn assess_create(
    arg: &mut RunArgStruct<'_, '_, '_>,
    start: DateTime<Utc>,
    explain: Value,
    action_value: Result<Box<dyn Scope>, chord::action::Error>,
) -> StepAssessStruct {
    let end = Utc::now();
    if let Err(e) = action_value.as_ref() {
        if !arg.catch_err() {
            error!(
                "step Err  {}\n{}\n<<<\n{}",
                arg.id(),
                e,
                explain_to_string(&explain),
            );
            return StepAssessStruct::new(
                arg.id().clone(),
                start,
                end,
                explain,
                StepState::Err(action_value.err().unwrap()),
                None,
            );
        } else {
            let map = arg.context_mut();
            map.insert("state".to_string(), Value::String("Err".to_string()));
            map.insert("value".to_string(), Value::String(e.to_string()));
            map.insert(
                "meta".to_string(),
                json!({
                    "start": start,
                    "end": end
                }),
            );
        }
    } else {
        let map = arg.context_mut();
        map.insert("state".to_string(), Value::String("Ok".to_string()));
        map.insert(
            "value".to_string(),
            action_value.as_ref().unwrap().as_value().clone(),
        );
        map.insert(
            "meta".to_string(),
            json!({
                "start": start,
                "end": end
            }),
        );
    }

    let then = assert_and_then(arg);
    return match then {
        Err(e) => {
            error!(
                "step Err  {}\n{}\n<<<\n{}",
                arg.id(),
                &e,
                explain_to_string(&explain)
            );
            StepAssessStruct::new(
                arg.id().clone(),
                start,
                end,
                explain,
                StepState::Err(Box::new(e)),
                None,
            )
        }
        Ok((ar, at)) => {
            if ar {
                info!("step Ok   {}", arg.id());
                StepAssessStruct::new(
                    arg.id().clone(),
                    start,
                    end,
                    explain,
                    StepState::Ok(action_value.unwrap()),
                    at,
                )
            } else {
                warn!(
                    "step Fail {}\n{}\n<<<\n{}",
                    arg.id(),
                    to_string_pretty(action_value.as_ref().unwrap().as_value())
                        .unwrap_or("".to_string()),
                    explain_to_string(&explain)
                );
                StepAssessStruct::new(
                    arg.id().clone(),
                    start,
                    end,
                    explain,
                    StepState::Fail(action_value.unwrap()),
                    None,
                )
            }
        }
    };
}

fn assert_and_then(arg: &RunArgStruct<'_, '_, '_>) -> Result<(bool, Option<StepThen>), Error> {
    let assert_success = value_assert(arg, arg.assert()).unwrap_or_else(|e| {
        debug!("step assert Err {}", e);
        false
    });
    return if !assert_success {
        Ok((false, None))
    } else {
        Ok((true, choose_then(arg)?))
    };
}

fn value_assert(
    arg: &RunArgStruct<'_, '_, '_>,
    condition: Option<&str>,
) -> Result<bool, TemplateRenderError> {
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
    for (idx, then) in then_vec.unwrap().iter().enumerate() {
        let cond: Option<&Value> = then.get("if");
        if cond.is_none()
            || cond.unwrap().as_str().is_none()
            || value_assert(arg, cond.unwrap().as_str())
                .map_err(|e| Render(format!("then.{}.if", idx), e))?
        {
            let reg = then.get("reg");
            let reg = if reg.is_none() {
                None
            } else if let Value::Object(m) = reg.unwrap() {
                Some(
                    arg.render_object(m)
                        .map_err(|e| Render(format!("then.{}.reg", idx), e))?,
                )
            } else {
                None
            };

            let goto = then.get("goto");
            let goto = if goto.is_none() {
                None
            } else if let Value::String(goto) = goto.unwrap() {
                let goto = arg
                    .render_str(goto.as_str())
                    .map_err(|e| Render(format!("then.{}.goto", idx), e))?;
                Some(goto.as_str().map(|s| s.to_string()).ok_or_else(|| {
                    ValueUnexpected(format!("then.{}.goto", idx), goto.to_string())
                })?)
            } else {
                None
            };

            return Ok(Some(StepThen::new(reg, goto)));
        }
    }
    return Ok(None);
}

fn assert(arg: &RunArgStruct<'_, '_, '_>, condition: &str) -> Result<bool, TemplateRenderError> {
    let assert_tpl = format!("{{{{{condition}}}}}", condition = condition);
    let assert_result = arg.render_str(assert_tpl.as_str())?;
    Ok(assert_result == "true")
}

fn explain_to_string(explain: &Value) -> String {
    if explain.is_string() {
        return explain.as_str().unwrap().to_string();
    } else {
        to_string_pretty(&explain).unwrap_or("".to_string())
    }
}
