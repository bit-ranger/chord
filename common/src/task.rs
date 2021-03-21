use chrono::{DateTime, Utc};

use crate::error::Error;
use crate::case::CaseAssess;

pub enum TaskState {
    Ok(Vec<Box<dyn CaseAssess>>),
    Err(Error),
    Fail(Vec<Box<dyn CaseAssess>>)
}

impl TaskState{

    #[allow(dead_code)]
    pub fn is_ok(&self) -> bool{
        match self {
            TaskState::Ok(_) => true,
            _ => false
        }
    }
}

pub trait TaskAssess{

    fn id(&self) -> &str;

    fn start(&self) -> DateTime<Utc>;

    fn end(&self) -> DateTime<Utc>;

    fn state(&self) -> &TaskState;
}


