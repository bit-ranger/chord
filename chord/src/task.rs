use std::fmt::Display;

use chrono::{DateTime, Utc};

use crate::error::Error;

pub trait TaskId: Sync + Send + Display {
    fn task(&self) -> &str;

    fn exec_id(&self) -> &str;
}

#[derive(Clone)]
pub enum TaskState {
    Ok,
    Fail,
    Err(Error),
}

impl TaskState {
    #[allow(dead_code)]
    pub fn is_ok(&self) -> bool {
        match self {
            TaskState::Ok => true,
            _ => false,
        }
    }
}

pub trait TaskAssess: Sync + Send {
    fn id(&self) -> &dyn TaskId;

    fn start(&self) -> DateTime<Utc>;

    fn end(&self) -> DateTime<Utc>;

    fn state(&self) -> &TaskState;
}
