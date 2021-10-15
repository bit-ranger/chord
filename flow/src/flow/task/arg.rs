use std::fmt::{Display, Formatter};

use chord::task::TaskId;

#[derive(Debug, Clone)]
pub struct TaskIdSimple {
    exec_id: String,
    task: String,
}

impl TaskIdSimple {
    pub fn new(exec_id: String, task_id: String) -> TaskIdSimple {
        TaskIdSimple {
            exec_id,
            task: task_id,
        }
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
