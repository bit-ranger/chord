pub use async_trait::async_trait;

use crate::error::Error;
use crate::value::Value;

#[async_trait]
pub trait CaseStore: Sync + Send {
    async fn create(&self, name: &str) -> Result<Box<dyn CaseLoad>, Error>;
}

#[async_trait]
pub trait CaseLoad: Sync + Send {
    async fn load(&mut self, size: usize) -> Result<Vec<(String, Value)>, Error>;
}
