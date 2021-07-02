use std::fmt::Display;

use chrono::{DateTime, Utc};

use crate::error::Error;
use crate::step::StepAssess;
use crate::task::TaskId;
use crate::value::Value;

pub trait CaseId: Sync + Send + Display {
    fn case(&self) -> &str;

    fn exec_id(&self) -> &str;

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

impl Drop for CaseState {
    fn drop(&mut self) {
        if let CaseState::Ok(vec) | CaseState::Fail(vec) = self {
            // last step first drop
            let _ = vec.pop();
        }
    }
}
