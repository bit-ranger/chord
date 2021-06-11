use chrono::{DateTime, Utc};

use crate::flow::case::arg::CaseIdStruct;
use chord::case::{CaseAssess, CaseId, CaseState};
use std::rc::Rc;

pub struct CaseAssessStruct {
    id: Rc<CaseIdStruct>,
    start: DateTime<Utc>,
    end: DateTime<Utc>,
    state: CaseState,
}

impl CaseAssessStruct {
    pub fn new(
        id: Rc<CaseIdStruct>,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        state: CaseState,
    ) -> CaseAssessStruct {
        CaseAssessStruct {
            id,
            start,
            end,
            state,
        }
    }
}

impl CaseAssess for CaseAssessStruct {
    fn id(&self) -> &dyn CaseId {
        self.id.as_ref()
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

unsafe impl Send for CaseAssessStruct {}

unsafe impl Sync for CaseAssessStruct {}
