use chrono::{DateTime, Utc};

use crate::action::{RunId, Scope};
use crate::error::Error;

pub enum StepState {
    Ok(Box<dyn Scope>),
    Fail(Box<dyn Scope>),
    Err(Error),
}

pub trait StepAssess: Sync + Send {
    fn id(&self) -> &dyn RunId;

    fn start(&self) -> DateTime<Utc>;

    fn end(&self) -> DateTime<Utc>;

    fn state(&self) -> &StepState;
}
