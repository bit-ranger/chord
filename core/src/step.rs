use chrono::{DateTime, Utc};

use crate::action::{Asset, Id};
use crate::collection::TailDropVec;
use crate::value::Value;

pub enum ActionState {
    Ok(Asset),
    Err(crate::action::Error),
}

impl ActionState {
    pub fn is_ok(&self) -> bool {
        match self {
            ActionState::Ok(_) => true,
            _ => false,
        }
    }

    pub fn is_err(&self) -> bool {
        match self {
            ActionState::Err(_) => true,
            _ => false,
        }
    }
}

pub trait ActionAsset: Sync + Send {
    fn id(&self) -> &str;

    fn start(&self) -> DateTime<Utc>;

    fn end(&self) -> DateTime<Utc>;

    fn explain(&self) -> &Value;

    fn state(&self) -> &ActionState;
}


pub enum StepState {
    Ok(TailDropVec<Box<dyn ActionAsset>>),
    Fail(TailDropVec<Box<dyn ActionAsset>>),
}

impl StepState {
    pub fn is_ok(&self) -> bool {
        match self {
            StepState::Ok(_) => true,
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

pub trait StepAsset: Sync + Send {
    fn id(&self) -> &dyn Id;

    fn start(&self) -> DateTime<Utc>;

    fn end(&self) -> DateTime<Utc>;

    fn state(&self) -> &StepState;
}
