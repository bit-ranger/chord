use chrono::{DateTime, Utc};

use chord_core::action::RunId;
use chord_core::collection::TailDropVec;
use chord_core::step::{StepAssess, StepState};
use chord_core::value::{Map, Value};

use crate::flow::step::arg::RunIdStruct;

pub struct ActionAssessStruct {
    aid: String,
    explain: Value,
    state: StepState,
}

impl ActionAssessStruct {
    pub fn new(aid: String, explain: Value, state: StepState) -> ActionAssessStruct {
        ActionAssessStruct {
            aid,
            explain,
            state,
        }
    }

    pub fn id(&self) -> &str {
        &self.aid
    }

    pub fn explain(&self) -> &Value {
        &self.explain
    }

    pub fn state(&self) -> &StepState {
        &self.state
    }
}

pub struct StepAssessStruct {
    id: RunIdStruct,
    start: DateTime<Utc>,
    end: DateTime<Utc>,
    #[allow(dead_code)]
    action_assess_vec: TailDropVec<ActionAssessStruct>,
    explain: Value,
    state: StepState,
}

impl StepAssessStruct {
    pub fn new(
        id: RunIdStruct,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        mut action_assess_vec: Vec<ActionAssessStruct>,
    ) -> StepAssessStruct {
        let mut em = Map::new();
        let mut sm = Map::new();

        for ast in action_assess_vec.iter() {
            em.insert(ast.aid.to_string(), ast.explain.clone());
            if let StepState::Ok(s) = ast.state() {
                sm.insert(ast.id().to_string(), s.as_value().clone());
            } else if let StepState::Err(e) = ast.state() {
                sm.insert(ast.id().to_string(), Value::String(e.to_string()));
            }
        }

        let explain = Value::Object(em);

        let state = if action_assess_vec.is_empty() {
            StepState::Ok(Box::new(Value::Object(Map::new())))
        } else {
            let last_is_err = (&action_assess_vec).last().unwrap().state.is_err();
            if last_is_err {
                action_assess_vec.pop().unwrap().state
            } else {
                StepState::Ok(Box::new(Value::Object(sm)))
            }
        };

        StepAssessStruct {
            id,
            start,
            end,
            action_assess_vec: TailDropVec::from(action_assess_vec),
            explain,
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

    fn explain(&self) -> &Value {
        &self.explain
    }

    fn state(&self) -> &StepState {
        &self.state
    }
}
