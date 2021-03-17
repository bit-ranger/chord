use chrono::{DateTime, Utc};

use crate::model::error::Error;
use crate::model::point::{PointResult};

pub type CaseResult = Result<Box<dyn CaseAssess>, Error>;

pub trait CaseAssess {

    fn id(&self) -> usize;

    fn start(&self) -> DateTime<Utc>;

    fn end(&self) -> DateTime<Utc>;

    fn state(&self) -> &CaseState;

    fn result(&self) -> &Vec<(String, PointResult)>;
}

#[derive(Debug)]
pub enum CaseState {
    Ok,
    PointError(Error),
    PointFailure
}

impl CaseState {

    pub fn is_ok(&self) -> bool{
        match self {
            CaseState::Ok => true,
            _ => false
        }
    }
}



