use std::sync::Arc;

pub use async_trait::async_trait;
pub use chrono::{DateTime, Utc};

use crate::case::CaseAsset;
use crate::flow::Flow;
use crate::task::{StageAsset, TaskAsset, TaskId};

pub type Error = Box<dyn std::error::Error + Sync + Send>;

#[async_trait]
pub trait JobReporter: Sync + Send {
    async fn task(
        &self,
        task_id: Arc<dyn TaskId>,
        flow: Arc<Flow>,
    ) -> Result<Box<dyn TaskReporter>, Error>;
}

#[async_trait]
pub trait TaskReporter: Sync + Send {
    async fn stage(&self, stage_id: &str) -> Result<Box<dyn StageReporter>, Error>;

    async fn start(&mut self, time: DateTime<Utc>) -> Result<(), Error>;

    async fn end(&mut self, task_asset: &dyn TaskAsset) -> Result<(), Error>;
}

#[async_trait]
pub trait StageReporter: Sync + Send {
    async fn start(&mut self, time: DateTime<Utc>) -> Result<(), Error>;

    async fn report(&mut self, case_asset_vec: &Vec<Box<dyn CaseAsset>>) -> Result<(), Error>;

    async fn end(&mut self, task_asset: &dyn StageAsset) -> Result<(), Error>;
}
