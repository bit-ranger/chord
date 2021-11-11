use crate::case::CaseAssess;
use crate::flow::Flow;
use crate::task::TaskAssess;
use async_std::sync::Arc;
pub use async_trait::async_trait;
pub use chrono::{DateTime, Utc};

pub enum Error {}

#[async_trait]
pub trait Report: Sync + Send {
    async fn start(&mut self, time: DateTime<Utc>, flow: Arc<Flow>) -> Result<(), Error>;

    async fn report(&mut self, case_assess_vec: &Vec<Box<dyn CaseAssess>>) -> Result<(), Error>;

    async fn end(&mut self, task_assess: &dyn TaskAssess) -> Result<(), Error>;
}
