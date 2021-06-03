use std::fmt::Display;

use chrono::{DateTime, Utc};
use lazy_static::lazy_static;

use crate::case::CaseAssess;
use crate::error::Error;
use regex::Regex;

lazy_static! {
    pub static ref TASK_ID_PATTERN: Regex = Regex::new(r"^[\w]+$").unwrap();
}

pub trait TaskId: Sync + Send + Display {
    fn task_id(&self) -> &str;

    fn exec_id(&self) -> &str;
}

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

unsafe impl Send for TaskState {}

unsafe impl Sync for TaskState {}

pub trait TaskAssess: Sync + Send {
    fn id(&self) -> &dyn TaskId;

    fn start(&self) -> DateTime<Utc>;

    fn end(&self) -> DateTime<Utc>;

    fn state(&self) -> &TaskState;
}
