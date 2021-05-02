use async_std::sync::Arc;

use chord_common::point::PointRunner;
pub use task::Runner;
pub use task::TASK_ID;
pub use case::CASE_ID;

use crate::model::app::{AppContext, AppContextStruct};

mod task;
mod case;
mod point;


pub async fn create_app_context(pt_runner: Box<dyn PointRunner>) -> Arc<dyn AppContext> {
    Arc::new(AppContextStruct::<'_>::new(pt_runner))
}

