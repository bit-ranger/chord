use chrono::{DateTime, Utc};

use chord_common::task::{TaskAssess, TaskState, TaskId};
use std::sync::Arc;
use crate::flow::task::arg::TaskIdStruct;

pub struct TaskAssessStruct {
    id: Arc<TaskIdStruct>,
    start: DateTime<Utc>,
    end: DateTime<Utc>,
    state: TaskState,
}

impl TaskAssessStruct {
    pub fn new(
        id: Arc<TaskIdStruct>,
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

unsafe impl Send for TaskAssessStruct {}

unsafe impl Sync for TaskAssessStruct {}
