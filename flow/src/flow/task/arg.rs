use chord_common::error::Error;
use chord_common::rerr;
use chord_common::task::TaskId;
use chord_common::task::TASK_ID_PATTERN;
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone)]
pub struct TaskIdSimple {
    exec_id: String,
    task_id: String,
}

impl TaskIdSimple {
    pub fn new(exec_id: String, task_id: String) -> Result<TaskIdSimple, Error> {
        if !TASK_ID_PATTERN.is_match(task_id.as_str()) {
            return rerr!("task", format!("invalid task_id {}", task_id));
        }

        Ok(TaskIdSimple { exec_id, task_id })
    }
}

impl TaskId for TaskIdSimple {
    fn task_id(&self) -> &str {
        self.task_id.as_str()
    }

    fn exec_id(&self) -> &str {
        self.exec_id.as_str()
    }
}
unsafe impl Send for TaskIdSimple {}
unsafe impl Sync for TaskIdSimple {}

impl Display for TaskIdSimple {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.write_str(format!("{}::{}", self.exec_id, self.task_id).as_str())
    }
}
