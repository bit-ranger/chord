use std::error::Error;

use async_std::sync::Arc;
pub use async_trait::async_trait;
pub use chrono::{DateTime, Utc};

use crate::case::CaseAssess;
use crate::flow::Flow;
use crate::task::TaskAssess;

#[async_trait]
pub trait Report: Sync + Send {
    async fn start(&mut self, time: DateTime<Utc>, flow: Arc<Flow>) -> Result<(), Box<dyn Error>>;

    async fn report(
        &mut self,
        case_assess_vec: &Vec<Box<dyn CaseAssess>>,
    ) -> Result<(), Box<dyn Error>>;

    async fn end(&mut self, task_assess: &dyn TaskAssess) -> Result<(), Box<dyn Error>>;
}
