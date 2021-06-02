use chrono::{DateTime, Utc};

use crate::case::CaseAssess;
use crate::error::Error;
use std::fmt::Display;


pub trait TaskId: Sync + Send + Display{

    fn task_id(&self) -> &str;

    fn exec_id(&self) -> &str;
}


pub enum TaskState {
    Ok(Vec<Box<dyn CaseAssess>>),
    Err(Error),
    Fail(Vec<Box<dyn CaseAssess>>),
}

impl TaskState {
    #[allow(dead_code)]
    pub fn is_ok(&self) -> bool {
        match self {
            TaskState::Ok(_) => true,
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


