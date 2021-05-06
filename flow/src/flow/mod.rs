use async_std::sync::Arc;

use chord_common::point::{PointRunnerFactory};
pub use task::Runner;
pub use task::TASK_ID;
pub use case::CASE_ID;

use crate::model::app::{FlowContext, FlowContextStruct};

mod task;
mod case;
mod point;


pub async fn create_context(point_runner_factory: Box<dyn PointRunnerFactory>) -> Arc<dyn FlowContext> {
    Arc::new(FlowContextStruct::<'_>::new(point_runner_factory))
}

