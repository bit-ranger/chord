use std::fmt::Display;

pub use async_trait::async_trait;
use chrono::{DateTime, Utc};

use crate::case::CaseId;
use crate::error::Error;
use crate::value::Value;

pub type ActionValue = std::result::Result<Value, Error>;

pub trait StepId: Sync + Send + Display {
    fn step_id(&self) -> &str;

    fn case_id(&self) -> &dyn CaseId;
}

pub trait RunArg: Sync + Send {
    fn id(&self) -> &dyn StepId;

    fn config(&self) -> &Value;

    fn render_str(&self, text: &str) -> Result<String, Error>;

    fn render_value(&self, text: &Value) -> Result<Value, Error>;
}

pub trait CreateArg: Sync + Send {
    fn id(&self) -> &dyn StepId;

    fn action(&self) -> &str;

    fn config(&self) -> &Value;

    fn render_str(&self, text: &str) -> Result<String, Error>;

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
pub trait Action: Sync + Send {
    async fn run(&self, arg: &dyn RunArg) -> ActionValue;
}

#[async_trait]
pub trait ActionFactory: Sync + Send {
    async fn create(&self, arg: &dyn CreateArg) -> Result<Box<dyn Action>, Error>;
}
