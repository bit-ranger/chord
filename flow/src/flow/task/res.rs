use std::sync::Arc;

use chrono::{DateTime, Utc};

use chord_core::task::{TaskAssess, TaskId, TaskState};

use crate::flow::task::arg::TaskIdSimple;

pub struct TaskAssessStruct {
    id: Arc<TaskIdSimple>,
    start: DateTime<Utc>,
    end: DateTime<Utc>,
    state: TaskState,
}

impl TaskAssessStruct {
    pub fn new(
        id: Arc<TaskIdSimple>,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        state: TaskState,
    ) -> TaskAssessStruct {
        TaskAssessStruct {
            id,
            start,
            end,
            state,
        }
    }
}

impl TaskAssess for TaskAssessStruct {
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
