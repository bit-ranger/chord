use std::error::Error;

pub use async_trait::async_trait;

use crate::value::Value;

#[async_trait]
pub trait CaseStore: Sync + Send {
    async fn create(&self, name: &str) -> Result<Box<dyn CaseLoad>, Box<dyn Error>>;
}

#[async_trait]
pub trait CaseLoad: Sync + Send {
    async fn load(&mut self, size: usize) -> Result<Vec<(String, Value)>, Box<dyn Error>>;
}
