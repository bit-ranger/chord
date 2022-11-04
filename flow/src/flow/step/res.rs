use chrono::{DateTime, Utc};

use chord_core::action::Id;
use chord_core::collection::TailDropVec;
use chord_core::step::{ActionAsset, ActionState, StepAsset, StepState};
use chord_core::value::Value;

use crate::flow::step::arg::IdStruct;

pub struct ActionAssetStruct {
    aid: String,
    start: DateTime<Utc>,
    end: DateTime<Utc>,
    explain: Value,
    state: ActionState,
}


impl ActionAssetStruct {
    pub fn new(aid: String, start: DateTime<Utc>, end: DateTime<Utc>, explain: Value, state: ActionState) -> ActionAssetStruct {
        ActionAssetStruct {
            aid,
            start,
            end,
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

impl ActionAsset for ActionAssetStruct {
    fn id(&self) -> &str {
        self.id()
    }

    fn start(&self) -> DateTime<Utc> {
        self.start
    }

    fn end(&self) -> DateTime<Utc> {
        self.end
    }

    fn explain(&self) -> &Value {
        self.explain()
    }

    fn state(&self) -> &ActionState {
        self.state()
    }
}


pub struct StepAssetStruct {
    id: IdStruct,
    start: DateTime<Utc>,
    end: DateTime<Utc>,
    state: StepState,
}

impl StepAssetStruct {
    pub fn new(
        id: IdStruct,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        action_asset_vec: Vec<ActionAssetStruct>,
    ) -> StepAssetStruct {
        let last_state_is_err = (&action_asset_vec).last().unwrap().state.is_err();

        let aav: Vec<Box<dyn ActionAsset>> = action_asset_vec
            .into_iter()
            .map(
                |a| Box::new(a) as Box<dyn ActionAsset>
            )
            .collect();


        let state = if last_state_is_err {
            StepState::Fail(TailDropVec::from(aav))
        } else {
            StepState::Ok(TailDropVec::from(aav))
        };

        StepAssetStruct {
            id,
            start,
            end,
            state,
        }
    }
}

impl StepAsset for StepAssetStruct {
    fn id(&self) -> &dyn Id {
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
