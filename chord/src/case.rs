use chrono::{DateTime, Utc};

use crate::error::Error;

use crate::step::StepAssess;
use crate::task::TaskId;
use std::fmt::Display;

pub trait CaseId: Sync + Send + Display {
    fn case_id(&self) -> usize;

    fn task_id(&self) -> &dyn TaskId;
}

pub trait CaseAssess: Sync + Send {
    fn id(&self) -> &dyn CaseId;

    fn start(&self) -> DateTime<Utc>;

    fn end(&self) -> DateTime<Utc>;

    fn state(&self) -> &CaseState;
}

pub enum CaseState {
    Ok(Vec<Box<dyn StepAssess>>),
    Err(Error),
    Fail(Vec<Box<dyn StepAssess>>),
}

impl CaseState {
    pub fn is_ok(&self) -> bool {
        match self {
            CaseState::Ok(_) => true,
            _ => false,
        }
    }
}
