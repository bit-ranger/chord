use chord_common::task::TaskId;
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone)]
pub struct TaskIdStruct {
    exec_id: String,
    task_id: String
}

impl TaskIdStruct {

    pub fn new(exec_id: String, task_id: String) -> TaskIdStruct{
        TaskIdStruct {
            exec_id, task_id
        }
    }

}


impl TaskId for TaskIdStruct {

    fn task_id(&self) -> &str {
        self.task_id.as_str()
    }

    fn exec_id(&self) -> &str {
        self.exec_id.as_str()
    }
}
unsafe impl Send for TaskIdStruct {}
unsafe impl Sync for TaskIdStruct {}

impl Display for TaskIdStruct {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.write_str(format!("{}::{}", self.exec_id, self.task_id).as_str())
    }
}