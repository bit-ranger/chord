use chrono::{DateTime, Utc};

use crate::error::Error;

use crate::point::{PointAssess};


pub trait CaseAssess {

    fn id(&self) -> usize;

    fn start(&self) -> DateTime<Utc>;

    fn end(&self) -> DateTime<Utc>;

    fn state(&self) -> &CaseState;
}

pub enum CaseState {
    Ok(Vec<Box<dyn PointAssess>>),
    Err(Error),
    Fail(Vec<Box<dyn PointAssess>>)
}

impl CaseState {

    pub fn is_ok(&self) -> bool{
        match self {
            CaseState::Ok(_) => true,
            _ => false
        }
    }
}



