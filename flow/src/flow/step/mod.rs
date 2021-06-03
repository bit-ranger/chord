use chrono::Utc;

use chord_common::error::Error;
use chord_common::step::{StepRunner, StepValue};
use res::StepAssessStruct;

use crate::flow::step::arg::RunArgStruct;
use crate::model::app::FlowContext;
use async_std::future::timeout;
use chord_common::step::StepState;
use log::trace;

pub mod arg;
pub mod res;

pub async fn run(
    _: &dyn FlowContext,
    arg: &RunArgStruct<'_, '_, '_, '_>,
    runner: &dyn StepRunner,
) -> StepAssessStruct {
    trace!("step start {}", arg.id());
    let start = Utc::now();
    let future = runner.run(arg);
    let timeout_value = timeout(arg.timeout(), future).await;
    let value = match timeout_value {
        Ok(v) => v,
        Err(_) => {
            return StepAssessStruct::new(
                arg.id().clone(),
                start,
                Utc::now(),
                StepState::Err(Error::new("002", "timeout")),
            );
        }
    };

    return match value {
        StepValue::Ok(json) => {
            StepAssessStruct::new(arg.id().clone(), start, Utc::now(), StepState::Ok(json))
        }
        StepValue::Err(e) => {
            StepAssessStruct::new(arg.id().clone(), start, Utc::now(), StepState::Err(e))
        }
    };
}
