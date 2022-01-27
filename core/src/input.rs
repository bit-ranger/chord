pub use async_trait::async_trait;

use crate::flow::Flow;
use crate::task::TaskId;
use crate::value::Value;
use std::sync::Arc;

pub type Error = Box<dyn std::error::Error + Sync + Send>;

#[async_trait]
pub trait JobLoader: Sync + Send {
    async fn create(
        &self,
        task_id: Arc<dyn TaskId>,
        flow: Arc<Flow>,
    ) -> Result<Box<dyn TaskLoader>, Error>;
}

#[async_trait]
pub trait TaskLoader: Sync + Send {
    async fn create(&self, stage_id: &str) -> Result<Box<dyn StageLoader>, Error>;
}

#[async_trait]
pub trait StageLoader: Sync + Send {
    async fn load(&mut self, size: usize) -> Result<Vec<(String, Value)>, Error>;
}
