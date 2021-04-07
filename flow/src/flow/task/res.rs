use chrono::{DateTime, Utc};

use chord_common::task::{TaskState, TaskAssess};

pub struct TaskAssessStruct {
    id: String,
    start: DateTime<Utc>,
    end: DateTime<Utc>,
    state: TaskState
}

impl TaskAssessStruct {

    pub fn new(id: &str,
               start: DateTime<Utc>,
               end: DateTime<Utc>,
               state: TaskState
    ) -> TaskAssessStruct {
        TaskAssessStruct {
            id: String::from(id),
            start,
            end,
            state
        }
    }


}

impl TaskAssess for TaskAssessStruct {
    fn id(&self) -> &str {
        &self.id
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

unsafe impl Send for TaskAssessStruct
{
}

unsafe impl Sync for TaskAssessStruct
{
}
