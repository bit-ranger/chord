use std::cell::RefCell;

use async_std::sync::Arc;
use async_std::task_local;

use chord::step::StepRunnerFactory;
pub use task::arg::TaskIdSimple;
pub use task::TaskRunner;

use crate::model::app::{Context, FlowContextStruct};

mod case;
mod step;
mod task;

task_local! {
    pub static CTX_ID: RefCell<String> = RefCell::new(String::new());
}

pub async fn context_create(step_runner_factory: Box<dyn StepRunnerFactory>) -> Arc<dyn Context> {
    Arc::new(FlowContextStruct::<'_>::new(step_runner_factory))
}
