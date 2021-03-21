use chrono::{DateTime, Utc};

use crate::error::Error;

use crate::point::{PointResult, PointAssess};

pub trait CaseAssess {

    fn id(&self) -> usize;

    fn start(&self) -> DateTime<Utc>;

    fn end(&self) -> DateTime<Utc>;

    fn state(&self) -> &CaseState;
}

#[derive(Debug, Clone)]
pub enum CaseState {
    Ok(Vec<dyn PointAssess>),
    Err(Error),
    PointFail(Vec<dyn PointAssess>)
}

impl CaseState {

    pub fn is_ok(&self) -> bool{
        match self {
            CaseState::Ok(_) => true,
            _ => false
        }
    }
}



