use chrono::{DateTime, Utc};

use chord::action::RunId;
use chord::step::{StepAssess, StepState};

use crate::flow::step::arg::RunIdStruct;
use chord::value::{Map, Value};

pub struct StepAssessStruct {
    id: RunIdStruct,
    start: DateTime<Utc>,
    end: DateTime<Utc>,
    explain: Value,
    state: StepState,
    then: Option<StepThen>,
}

pub struct StepThen {
    reg: Option<Map>,
    goto: Option<String>,
}

impl StepThen {
    pub fn reg(&self) -> Option<&Map> {
        self.reg.as_ref()
    }

    pub fn goto(&self) -> Option<&str> {
        self.goto.as_ref().map(|g| g.as_str())
    }

    pub fn new(reg: Option<Map>, goto: Option<String>) -> StepThen {
        StepThen { reg, goto }
    }
}

impl StepAssessStruct {
    pub fn new(
        id: RunIdStruct,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        explain: Value,
        state: StepState,
        then: Option<StepThen>,
    ) -> StepAssessStruct {
        StepAssessStruct {
            id,
            start,
            end,
            explain,
            state,
            then,
        }
    }

    pub fn then(&self) -> Option<&StepThen> {
        self.then.as_ref()
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
        &self.explain
    }

    fn state(&self) -> &StepState {
        &self.state
    }
}
