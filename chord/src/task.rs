use std::fmt::Display;

use chrono::{DateTime, Utc};
use std::error::Error;

pub trait TaskId: Sync + Send + Display {
    fn task(&self) -> &str;

    fn exec_id(&self) -> &str;
}

pub enum TaskState {
    Ok,
    Fail(String),
    Err(Box<dyn Error>),
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
