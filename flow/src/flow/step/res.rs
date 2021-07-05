use chrono::{DateTime, Utc};

use chord::action::RunId;
use chord::step::{StepAssess, StepState};

use crate::flow::step::arg::RunIdStruct;

pub struct StepAssessStruct {
    pub id: RunIdStruct,
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
    pub state: StepState,
}

impl StepAssessStruct {
    pub fn new(
        id: RunIdStruct,
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
    fn id(&self) -> &dyn RunId {
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
