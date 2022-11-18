use std::fmt::{Display, Formatter};
use std::sync::Arc;

use chord_core::task::{StageId, TaskId};

#[derive(Debug, Clone)]
pub struct TaskIdStruct {
    exec: String,
    task: String,
}

impl TaskIdStruct {
    pub fn new(exec_id: String, task_id: String) -> TaskIdStruct {
        TaskIdStruct {
            exec: exec_id,
            task: task_id,
        }
    }
}

impl TaskId for TaskIdStruct {
    fn task(&self) -> &str {
        self.task.as_str()
    }

    fn exec(&self) -> &str {
        self.exec.as_str()
    }
}

impl Display for TaskIdStruct {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.write_str(format!("{}-{}", self.exec, self.task).as_str())
    }
}


#[derive(Debug, Clone)]
pub struct StageIdStruct {
    task: Arc<TaskIdStruct>,
    stage: String,
    exec: String,
}

impl StageIdStruct {
    pub fn new(task: Arc<TaskIdStruct>,
               stage: String,
               exec: String) -> StageIdStruct {
        StageIdStruct {
            task,
            stage,
            exec,
        }
    }
}

impl StageId for StageIdStruct {
    fn task(&self) -> &dyn TaskId {
        self.task.as_ref()
    }

    fn stage(&self) -> &str {
        self.stage.as_str()
    }

    fn exec(&self) -> &str {
        self.exec.as_str()
    }
}

impl Display for StageIdStruct {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.write_str(format!("{}-{}-{}", self.task, self.stage, self.exec).as_str())
    }
}