pub use async_trait::async_trait;
pub use chrono::{DateTime, Utc};

use crate::case::CaseAssess;
use crate::task::TaskAssess;

pub type Error = Box<dyn std::error::Error + Sync + Send>;

#[async_trait]
pub trait Report: Sync + Send {
    async fn start(&mut self, time: DateTime<Utc>) -> Result<(), Error>;

    async fn report(&mut self, case_assess_vec: &Vec<Box<dyn CaseAssess>>) -> Result<(), Error>;

    async fn end(&mut self, task_assess: &dyn TaskAssess) -> Result<(), Error>;
}
