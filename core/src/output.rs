use std::sync::Arc;

pub use async_trait::async_trait;
pub use chrono::{DateTime, Utc};

use crate::case::CaseAssess;
use crate::flow::Flow;
use crate::task::{StageAssess, TaskAssess, TaskId};

pub type Error = Box<dyn std::error::Error + Sync + Send>;

#[async_trait]
pub trait JobReporter: Sync + Send {
    async fn create(
        &self,
        task_id: Arc<dyn TaskId>,
        flow: Arc<Flow>,
    ) -> Result<Box<dyn TaskReporter>, Error>;
}

#[async_trait]
pub trait TaskReporter: Sync + Send {
    async fn create(&self, stage_id: &str) -> Result<Box<dyn StageReporter>, Error>;

    async fn start(&mut self, time: DateTime<Utc>) -> Result<(), Error>;

    async fn end(&mut self, task_assess: &dyn TaskAssess) -> Result<(), Error>;
}

#[async_trait]
pub trait StageReporter: Sync + Send {
    async fn start(&mut self, time: DateTime<Utc>) -> Result<(), Error>;

    async fn report(&mut self, case_assess_vec: &Vec<Box<dyn CaseAssess>>) -> Result<(), Error>;

    async fn end(&mut self, task_assess: &dyn StageAssess) -> Result<(), Error>;
}
