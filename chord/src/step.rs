pub use async_trait::async_trait;
use chrono::{DateTime, Utc};

use crate::case::CaseId;
use crate::error::Error;
use crate::value::Value;
use lazy_static::lazy_static;
use regex::Regex;
use std::fmt::Display;

pub type StepValue = std::result::Result<Value, Error>;

lazy_static! {
    pub static ref POINT_ID_PATTERN: Regex = Regex::new(r"^[\w]+$").unwrap();
}

pub trait StepId: Sync + Send + Display {
    fn step_id(&self) -> &str;

    fn case_id(&self) -> &dyn CaseId;
}

pub trait RunArg: Sync + Send {
    fn id(&self) -> &dyn StepId;

    fn config(&self) -> &Value;

    fn render(&self, text: &str) -> Result<String, Error>;
}

pub trait CreateArg: Sync + Send {
    fn id(&self) -> &dyn StepId;

    fn kind(&self) -> &str;

    fn config(&self) -> &Value;

    fn render(&self, text: &str) -> Result<String, Error>;

    fn is_task_shared(&self, text: &str) -> bool;
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

#[async_trait]
pub trait StepRunner: Sync + Send {
    async fn run(&self, arg: &dyn RunArg) -> StepValue;
}

#[async_trait]
pub trait StepRunnerFactory: Sync + Send {
    async fn create(&self, arg: &dyn CreateArg) -> Result<Box<dyn StepRunner>, Error>;
}
