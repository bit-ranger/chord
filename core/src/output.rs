use std::sync::Arc;

pub use async_trait::async_trait;
pub use chrono::{DateTime, Utc};

use crate::case::CaseAssess;
use crate::flow::Flow;
use crate::task::{TaskAssess, TaskId};

pub type Error = Box<dyn std::error::Error + Sync + Send>;

#[async_trait]
pub trait Report: Sync + Send {
    async fn start(&mut self, time: DateTime<Utc>) -> Result<(), Error>;

    async fn report(&mut self, case_assess_vec: &Vec<Box<dyn CaseAssess>>) -> Result<(), Error>;

    async fn end(&mut self, task_assess: &dyn TaskAssess) -> Result<(), Error>;
}

#[async_trait]
pub trait Factory: Sync + Send {
    async fn create(
        &self,
        task_id: Arc<dyn TaskId>,
        flow: Arc<Flow>,
    ) -> Result<Box<dyn Report>, Error>;
}
