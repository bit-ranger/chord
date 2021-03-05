use std::collections::BTreeMap;

use serde_json::Value;

use crate::model::case::{CaseContext, CaseResult};

#[derive(Debug)]
pub struct TaskContext {
    data: Vec<BTreeMap<String,String>>,
    config: Value
}


impl TaskContext {

    pub fn new(config: Value, data: Vec<BTreeMap<String,String>>) -> TaskContext {
        let context = TaskContext {
            config,
            data
        };
        return context;
    }


    pub fn create_case(self: &TaskContext) -> Vec<CaseContext<'_, '_>> {
        return self.data.iter()
            .enumerate()
            .map(|(idx,_)| {
                CaseContext::new(
                    &self.config,
                    &self.data[idx]
                )
            })
            .collect();
    }

}

pub type TaskResult = std::result::Result<Vec<CaseResult>, ()>;
