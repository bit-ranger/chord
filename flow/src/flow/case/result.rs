use chrono::{DateTime, Utc};

use common::case::{CaseState, CaseAssess};

pub struct CaseAssessStruct {
    id: usize,
    start: DateTime<Utc>,
    end: DateTime<Utc>,
    state: CaseState
}

impl CaseAssessStruct {

    pub fn new(id: usize,
               start: DateTime<Utc>,
               end: DateTime<Utc>,
               state: CaseState
    ) -> CaseAssessStruct {
        CaseAssessStruct {
            id,start,end,state
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
}



