use chrono::{DateTime, Utc};

use crate::action::{Id, Data, Asset};
use crate::value::Value;

pub enum StepState {
    Ok(Value),
    Err(crate::action::Error),
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
}

pub trait StepAsset: Sync + Send {
    fn id(&self) -> &dyn Id;

    fn start(&self) -> DateTime<Utc>;

    fn end(&self) -> DateTime<Utc>;

    fn explain(&self) -> &Value;

    fn state(&self) -> &StepState;
}
