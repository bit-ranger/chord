use chrono::Utc;

use chord_common::error::Error;
use chord_common::point::{PointValue, PointRunner};
use res::PointAssessStruct;

use crate::flow::point::arg::PointArgStruct;
use crate::model::app::FlowContext;
use chord_common::point::{PointState};
use async_std::future::timeout;
use log::{trace};

pub mod arg;
pub mod res;


pub async fn run(_: &dyn FlowContext, arg: &PointArgStruct<'_, '_, '_, '_>, runner: &dyn PointRunner) -> PointAssessStruct
{
    trace!("point start {}", arg.id());
    let start = Utc::now();
    let future = runner.run(arg);
    let timeout_value = timeout(arg.timeout(), future).await;
    let value = match timeout_value {
        Ok(v) => v,
        Err(_) => {
            return PointAssessStruct::new(arg.id(), start, Utc::now(), PointState::Err(Error::new("002", "timeout")));
        }
    };

    return match value {
        PointValue::Ok(json) => {
            PointAssessStruct::new(arg.id(), start, Utc::now(), PointState::Ok(json))
        },
        PointValue::Err(e) => {
            PointAssessStruct::new(arg.id(), start, Utc::now(), PointState::Err(e))
        }
    };
}


