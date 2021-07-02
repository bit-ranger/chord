use std::fmt::Display;

use chrono::{DateTime, Utc};

use crate::action::Scope;
use crate::case::CaseId;
use crate::error::Error;

pub trait StepId: Sync + Send + Display {
    fn step(&self) -> &str;

    fn case_id(&self) -> &dyn CaseId;
}

pub enum StepState {
    Ok(Box<dyn Scope>),
    Fail(Box<dyn Scope>),
    Err(Error),
}

pub trait StepAssess: Sync + Send {
    fn id(&self) -> &dyn StepId;

    fn start(&self) -> DateTime<Utc>;

    fn end(&self) -> DateTime<Utc>;

    fn state(&self) -> &StepState;
}
