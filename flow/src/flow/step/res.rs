use chrono::{DateTime, Utc};

use chord_core::action::RunId;
use chord_core::collection::TailDropVec;
use chord_core::step::{StepAssess, StepState};
use chord_core::value::{Map, Value};

use crate::flow::step::arg::RunIdStruct;

pub struct StepAssessStruct {
    id: RunIdStruct,
    start: DateTime<Utc>,
    end: DateTime<Utc>,
    action_assess_vec: TailDropVec<ActionAssessStruct>,
}

pub struct ActionAssessStruct {
    start: DateTime<Utc>,
    end: DateTime<Utc>,
    explain: Value,
    state: StepState,
}

impl ActionAssessStruct {
    pub fn new(
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        explain: Value,
        state: StepState,
    ) -> ActionAssessStruct {
        ActionAssessStruct {
            start,
            end,
            explain,
            state,
        }
    }
}

impl StepAssessStruct {
    pub fn new(
        id: RunIdStruct,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        action_assess_vec: Vec<ActionAssessStruct>,
    ) -> StepAssessStruct {
        StepAssessStruct {
            id,
            start,
            end,
            action_assess_vec: TailDropVec::from(action_assess_vec),
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

    fn explain(&self) -> &Value {
        &self.action_assess_vec.last().unwrap().explain
    }

    fn state(&self) -> &StepState {
        &self.action_assess_vec.last().unwrap().state
    }
}
