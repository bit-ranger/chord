use chrono::{DateTime, Utc};

use crate::action::{RunId, Scope};
use crate::value::Value;

pub enum StepState {
    Ok(Box<dyn Scope>),
    Fail(Box<dyn Scope>),
    Err(Box<dyn std::error::Error>),
}

impl StepState {
    pub fn is_ok(&self) -> bool {
        match self {
            StepState::Ok(_) => true,
            _ => false,
        }
    }

    pub fn is_err(&self) -> bool {
        match self {
            StepState::Err(_) => true,
            _ => false,
        }
    }

    pub fn is_fail(&self) -> bool {
        match self {
            StepState::Fail(_) => true,
            _ => false,
        }
    }
}

pub trait StepAssess: Sync + Send {
    fn id(&self) -> &dyn RunId;

    fn start(&self) -> DateTime<Utc>;

    fn end(&self) -> DateTime<Utc>;

    fn explain(&self) -> &Value;

    fn state(&self) -> &StepState;
}
