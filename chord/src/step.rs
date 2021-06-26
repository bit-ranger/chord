use std::fmt::Display;

pub use async_trait::async_trait;
use chrono::{DateTime, Utc};

use crate::case::CaseId;
use crate::error::Error;
use crate::value::Value;

pub trait StepId: Sync + Send + Display {
    fn step(&self) -> &str;

    fn case_id(&self) -> &dyn CaseId;
}

#[derive(Debug, Clone)]
pub enum StepState {
    Ok(Value),
    Fail(Value),
    Err(Error),
}

pub trait StepAssess: Sync + Send {
    fn id(&self) -> &dyn StepId;

    fn start(&self) -> DateTime<Utc>;

    fn end(&self) -> DateTime<Utc>;

    fn state(&self) -> &StepState;
}
