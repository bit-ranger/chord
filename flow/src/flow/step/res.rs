use chrono::{DateTime, Utc};

use chord::action::RunId;
use chord::step::{StepAssess, StepState};

use crate::flow::step::arg::RunIdStruct;

pub struct StepAssessStruct {
    id: RunIdStruct,
    start: DateTime<Utc>,
    end: DateTime<Utc>,
    state: StepState,
    goto: Option<String>,
}

impl StepAssessStruct {
    pub fn new(
        id: RunIdStruct,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        state: StepState,
        goto: Option<String>,
    ) -> StepAssessStruct {
        StepAssessStruct {
            id,
            start,
            end,
            state,
            goto,
        }
    }

    pub fn get_goto(&self) -> Option<&str> {
        self.goto.as_ref().map(|g| g.as_str())
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
