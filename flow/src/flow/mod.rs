use std::cell::RefCell;

use async_std::sync::Arc;
use async_std::task_local;

use chord_common::step::StepRunnerFactory;
pub use task::arg::TaskIdStruct;
pub use task::Runner;

use crate::model::app::{FlowContext, FlowContextStruct};

mod case;
mod step;
mod task;

task_local! {
    pub static CTX_ID: RefCell<String> = RefCell::new(String::new());
}

pub async fn create_context(
    step_runner_factory: Box<dyn StepRunnerFactory>,
) -> Arc<dyn FlowContext> {
    Arc::new(FlowContextStruct::<'_>::new(step_runner_factory))
}
