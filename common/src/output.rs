use crate::case::CaseAssess;
use crate::error::Error;
use crate::task::TaskAssess;
pub use async_trait::async_trait;
pub use chrono::{DateTime, Utc};

#[async_trait]
pub trait AssessReport: Sync + Send {
    async fn start(&mut self, time: DateTime<Utc>) -> Result<(), Error>;

    async fn report(&mut self, case_assess_vec: &Vec<Box<dyn CaseAssess>>) -> Result<(), Error>;

    async fn end(&mut self, task_assess: &dyn TaskAssess) -> Result<(), Error>;
}
