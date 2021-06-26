use std::fmt::{Display, Formatter};

use chord::rerr;
use chord::task::TaskId;
use chord::task::TASK_ID_PATTERN;
use chord::Error;

#[derive(Debug, Clone)]
pub struct TaskIdSimple {
    exec_id: String,
    task: String,
}

impl TaskIdSimple {
    pub fn new(exec_id: String, task_id: String) -> Result<TaskIdSimple, Error> {
        if !TASK_ID_PATTERN.is_match(task_id.as_str()) {
            return rerr!("task", format!("invalid task_id {}", task_id));
        }

        Ok(TaskIdSimple {
            exec_id,
            task: task_id,
        })
    }
}

impl TaskId for TaskIdSimple {
    fn task(&self) -> &str {
        self.task.as_str()
    }

    fn exec_id(&self) -> &str {
        self.exec_id.as_str()
    }
}

impl Display for TaskIdSimple {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.write_str(format!("{}-{}", self.exec_id, self.task).as_str())
    }
}
