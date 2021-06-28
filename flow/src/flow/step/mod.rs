use std::panic::AssertUnwindSafe;

use async_std::future::timeout;
use chrono::Utc;
use futures::FutureExt;
use log::trace;

use chord::action::{Action, ActionValue};
use chord::step::StepState;
use chord::Error;
use res::StepAssessStruct;

use crate::flow::step::arg::RunArgStruct;
use crate::model::app::Context;

pub mod arg;
pub mod res;

pub async fn run(
    _: &dyn Context,
    arg: &RunArgStruct<'_, '_, '_, '_>,
    action: &dyn Action,
) -> StepAssessStruct {
    trace!("step start {}", arg.id());
    let start = Utc::now();
    let future = AssertUnwindSafe(action.run(arg)).catch_unwind();
    let timeout_value = timeout(arg.timeout(), future).await;
    if let Err(_) = timeout_value {
        return StepAssessStruct::new(
            arg.id().clone(),
            start,
            Utc::now(),
            StepState::Err(Error::new("001", "timeout")),
        );
    }
    let unwind_value = timeout_value.unwrap();
    if let Err(_) = unwind_value {
        return StepAssessStruct::new(
            arg.id().clone(),
            start,
            Utc::now(),
            StepState::Err(Error::new("002", "unwind")),
        );
    }
    let action_value = unwind_value.unwrap();

    return match action_value {
        ActionValue::Ok(json) => {
            StepAssessStruct::new(arg.id().clone(), start, Utc::now(), StepState::Ok(json))
        }
        ActionValue::Err(e) => {
            StepAssessStruct::new(arg.id().clone(), start, Utc::now(), StepState::Err(e))
        }
    };
}
