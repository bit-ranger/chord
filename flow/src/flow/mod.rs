use async_std::sync::Arc;

pub use case::CASE_ID;
use chord_common::point::PointRunnerFactory;
pub use task::Runner;
pub use task::arg::TaskIdStruct;
pub use task::TASK_ID;

use crate::model::app::{FlowContext, FlowContextStruct};

mod case;
mod point;
mod task;

pub async fn create_context(
    point_runner_factory: Box<dyn PointRunnerFactory>,
) -> Arc<dyn FlowContext> {
    Arc::new(FlowContextStruct::<'_>::new(point_runner_factory))
}
