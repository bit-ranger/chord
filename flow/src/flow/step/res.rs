use chrono::{DateTime, Utc};

use chord_common::step::{StepAssess, StepState, StepId};
use crate::flow::step::arg::StepIdStruct;

pub struct StepAssessStruct {
    pub id: StepIdStruct,
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
    pub state: StepState,
}

impl StepAssessStruct {
    pub fn new(
        id: StepIdStruct,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        state: StepState,
    ) -> StepAssessStruct {
        StepAssessStruct {
            id,
            start,
            end,
            state,
        }
    }
}

impl StepAssess for StepAssessStruct {
    fn id(&self) -> &dyn StepId {
        &self.id
    }

    fn start(&self) -> DateTime<Utc> {
        self.start
    }

    fn end(&self) -> DateTime<Utc> {
        self.end
    }

    fn state(&self) -> &StepState {
        &self.state
    }
}

unsafe impl Send for StepAssessStruct {}

unsafe impl Sync for StepAssessStruct {}
