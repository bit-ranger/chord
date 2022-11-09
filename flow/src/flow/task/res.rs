use std::sync::Arc;

use chrono::{DateTime, Utc};

use chord_core::task::{StageAsset, StageState, TaskAsset, TaskId, TaskState};

use crate::flow::task::arg::TaskIdSimple;

pub struct TaskAssetStruct {
    id: Arc<TaskIdSimple>,
    start: DateTime<Utc>,
    end: DateTime<Utc>,
    state: TaskState,
}

impl TaskAssetStruct {
    pub fn new(
        id: Arc<TaskIdSimple>,
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
    id: String,
    start: DateTime<Utc>,
    end: DateTime<Utc>,
    state: StageState,
}

impl StageAssetStruct {
    pub fn new(
        id: String,
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
    fn id(&self) -> &str {
        self.id.as_str()
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
