use std::collections::BTreeMap;
use async_std::sync::Arc;
use serde_yaml::Value;

#[derive(Debug)]
pub struct TaskContext {
    data: Vec<BTreeMap<String,String>>,
    flow: Value
}


impl TaskContext {

    pub fn new(flow: Value, data: Vec<BTreeMap<String,String>>) -> TaskContext {
        TaskContext{
            flow,
            data
        }
    }

    pub fn split(self) -> Vec<CaseContext>{
        let rc = Arc::new(self);
        return rc.data.iter()
            .enumerate()
            .map(|(idx, _d)| {
                CaseContext{
                    task_context: rc.clone(),
                    data_index: idx
                }
            })
            .collect();
    }
}

#[derive(Debug)]
pub struct CaseContext {

    task_context: Arc<TaskContext>,
    data_index: usize

}








