use std::fmt::Display;

use chrono::{DateTime, Utc};

use crate::collection::TailDropVec;
use crate::step::StepAssess;
use crate::task::TaskId;
use crate::value::Value;

pub type Error = Box<dyn std::error::Error + Sync + Send>;

pub trait CaseId: Sync + Send + Display {
    fn case(&self) -> &str;

    fn exec_id(&self) -> &str;

    fn stage_id(&self) -> &str;

    fn task_id(&self) -> &dyn TaskId;
}

pub trait CaseAssess: Sync + Send {
    fn id(&self) -> &dyn CaseId;

    fn start(&self) -> DateTime<Utc>;

    fn end(&self) -> DateTime<Utc>;

    fn data(&self) -> &Value;

    fn state(&self) -> &CaseState;
}

pub enum CaseState {
    Ok(TailDropVec<Box<dyn StepAssess>>),
    Err(Box<Error>),
    Fail(TailDropVec<Box<dyn StepAssess>>),
}

impl CaseState {
    pub fn is_ok(&self) -> bool {
        match self {
            CaseState::Ok(_) => true,
            _ => false,
        }
    }
}
