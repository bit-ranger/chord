use std::sync::Arc;

use chrono::{DateTime, Utc};

use chord_core::task::{StageAsset, StageId, StageState, TaskAsset, TaskId, TaskState};

use crate::flow::task::arg::{StageIdStruct, TaskIdStruct};

pub struct TaskAssetStruct {
    id: Arc<TaskIdStruct>,
    start: DateTime<Utc>,
    end: DateTime<Utc>,
    state: TaskState,
}

impl TaskAssetStruct {
    pub fn new(
        id: Arc<TaskIdStruct>,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        state: TaskState,
    ) -> TaskAssetStruct {
        TaskAssetStruct {
            id,
            start,
            end,
            state,
        }
    }
}

impl TaskAsset for TaskAssetStruct {
    fn id(&self) -> &dyn TaskId {
        self.id.as_ref()
    }

    fn start(&self) -> DateTime<Utc> {
        self.start
    }

    fn end(&self) -> DateTime<Utc> {
        self.end
    }

    fn state(&self) -> &TaskState {
        &self.state
    }
}

pub struct StageAssetStruct {
    id: Arc<StageIdStruct>,
    start: DateTime<Utc>,
    end: DateTime<Utc>,
    state: StageState,
}

impl StageAssetStruct {
    pub fn new(
        id: Arc<StageIdStruct>,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        state: StageState,
    ) -> StageAssetStruct {
        StageAssetStruct {
            id,
            start,
            end,
            state,
        }
    }
}

impl StageAsset for StageAssetStruct {
    fn id(&self) -> &dyn StageId {
        self.id.as_ref()
    }

    fn start(&self) -> DateTime<Utc> {
        self.start
    }

    fn end(&self) -> DateTime<Utc> {
        self.end
    }

    fn state(&self) -> &StageState {
        &self.state
    }
}
