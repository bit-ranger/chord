use chrono::{DateTime, Utc};

use common::error::Error;

use crate::model::case::CaseResult;

pub type TaskResult = Result<Box<dyn TaskAssess>, Error>;

#[derive(Debug)]
pub enum TaskState {
    Ok,
    CaseError(Error),
    CaseFailure
}

impl TaskState{

    #[allow(dead_code)]
    pub fn is_ok(&self) -> bool{
        match self {
            TaskState::Ok => true,
            _ => false
        }
    }
}

pub trait TaskAssess{

    fn id(&self) -> &str;

    fn start(&self) -> DateTime<Utc>;

    fn end(&self) -> DateTime<Utc>;

    fn state(&self) -> &TaskState;

    fn result(&self) -> &Vec<(usize, CaseResult)>;
}


