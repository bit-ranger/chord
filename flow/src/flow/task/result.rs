use chrono::{DateTime, Utc};

use common::task::{TaskState, TaskAssess};
use common::case::CaseResult;


pub struct TaskResultStruct {
    result: Vec<(usize, CaseResult)>,
    id: String,
    start: DateTime<Utc>,
    end: DateTime<Utc>,
    state: TaskState
}

impl TaskResultStruct{

    pub fn new(result:Vec<(usize, CaseResult)>,
               id: &str,
               start: DateTime<Utc>,
               end: DateTime<Utc>,
               state: TaskState
    ) -> TaskResultStruct {
        TaskResultStruct {
            result,
            id: String::from(id),
            start,
            end,
            state
        }
    }


}

impl TaskAssess for TaskResultStruct{
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

    fn result(&self) -> &Vec<(usize, CaseResult)> {
        &self.result
    }
}

