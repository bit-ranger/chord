use crate::error::Error;
use crate::task::TaskAssess;
pub use async_trait::async_trait;

#[async_trait]
pub trait Report {
    async fn write(&mut self, task_assess: &dyn TaskAssess) -> Result<(), Error>;
}
