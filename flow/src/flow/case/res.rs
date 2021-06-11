use std::sync::Arc;

use chrono::{DateTime, Utc};

use chord::case::{CaseAssess, CaseId, CaseState};

use crate::flow::case::arg::CaseIdStruct;

pub struct CaseAssessStruct {
    id: Arc<CaseIdStruct>,
    start: DateTime<Utc>,
    end: DateTime<Utc>,
    state: CaseState,
}

impl CaseAssessStruct {
    pub fn new(
        id: Arc<CaseIdStruct>,
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
