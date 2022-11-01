use chrono::{DateTime, Utc};

use chord_core::action::{Asset, Id};
use chord_core::collection::TailDropVec;
use chord_core::step::{StepAsset, StepState};
use chord_core::value::{Map, Value};

use crate::flow::step::arg::IdStruct;

pub enum ActionState {
    Ok(Asset),
    Err(chord_core::action::Error),
}

impl ActionState {
    #[warn(dead_code)]
    pub fn is_ok(&self) -> bool {
        match self {
            ActionState::Ok(_) => true,
            _ => false,
        }
    }

    pub fn is_err(&self) -> bool {
        match self {
            ActionState::Err(_) => true,
            _ => false,
        }
    }
}

pub struct ActionAssessStruct {
    aid: String,
    explain: Value,
    state: ActionState,
}

impl ActionAssessStruct {
    pub fn new(aid: String, explain: Value, state: ActionState) -> ActionAssessStruct {
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

    pub fn state(&self) -> &ActionState {
        &self.state
    }
}

pub struct StepAssessStruct {
    id: IdStruct,
    start: DateTime<Utc>,
    end: DateTime<Utc>,
    #[allow(dead_code)]
    action_assess_vec: TailDropVec<ActionAssessStruct>,
    explain: Value,
    state: StepState,
}

impl StepAssessStruct {
    pub fn new(
        id: IdStruct,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        mut action_assess_vec: Vec<ActionAssessStruct>,
    ) -> StepAssessStruct {
        let mut em = Map::new();
        let mut sm = Map::new();

        for ast in action_assess_vec.iter() {
            em.insert(ast.aid.to_string(), ast.explain.clone());
            if let ActionState::Ok(s) = ast.state() {
                sm.insert(ast.id().to_string(), s.to_value());
            } else if let ActionState::Err(e) = ast.state() {
                sm.insert(ast.id().to_string(), Value::String(e.to_string()));
            }
        }

        let explain = Value::Object(em);

        let state = if action_assess_vec.is_empty() {
            StepState::Ok(Value::Object(Map::new()))
        } else {
            let last_state = &action_assess_vec.last().unwrap().state;
            if last_state.is_err() {
                if let ActionState::Err(e) = action_assess_vec.pop().unwrap().state {
                    StepState::Err(e)
                } else {
                    unreachable!()
                }
            } else {
                StepState::Ok(Value::Object(sm))
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

impl StepAsset for StepAssessStruct {
    fn id(&self) -> &dyn Id {
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
