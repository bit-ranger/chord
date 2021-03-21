use chrono::{DateTime, Utc};

use crate::error::Error;
use crate::case::CaseAssess;


#[derive(Debug, Clone)]
pub enum TaskState {
    Ok(Vec<dyn CaseAssess>),
    Err(Error),
    CaseFail(Vec<dyn CaseAssess>)
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


