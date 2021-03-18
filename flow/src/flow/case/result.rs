use chrono::{DateTime, Utc};

use crate::model::case::{CaseState, CaseAssess};
use crate::model::point::PointResult;

pub struct CaseAssessStruct {
    result: Vec<(String,PointResult)>,
    id: usize,
    start: DateTime<Utc>,
    end: DateTime<Utc>,
    state: CaseState
}

impl CaseAssessStruct {

    pub fn new(result:Vec<(String, PointResult)>,
               id: usize,
               start: DateTime<Utc>,
               end: DateTime<Utc>,
               state: CaseState
    ) -> CaseAssessStruct {
        CaseAssessStruct {
            result,id,start,end,state
        }
    }



}

impl CaseAssess for CaseAssessStruct {

    fn id(&self) -> usize {
        self.id
    }
    fn start(&self) -> DateTime<Utc> {
        self.start
    }
    fn end(&self) -> DateTime<Utc> {
        self.end
    }
    fn state(&self) -> &CaseState {
        &self.state
    }
    fn result(&self) -> &Vec<(String, PointResult)> {
        &self.result
    }
}



