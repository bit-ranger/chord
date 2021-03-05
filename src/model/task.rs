use std::collections::BTreeMap;

use serde_json::Value;

use crate::model::case::{CaseContextStruct, CaseResult};

#[derive(Debug)]
pub struct TaskContextStruct {
    data: Vec<BTreeMap<String,String>>,
    config: Value
}


impl TaskContextStruct {

    pub fn new(config: Value, data: Vec<BTreeMap<String,String>>) -> TaskContextStruct {
        let context = TaskContextStruct {
            config,
            data
        };
        return context;
    }


    pub fn create_case(self: &TaskContextStruct) -> Vec<CaseContextStruct<'_, '_>> {
        return self.data.iter()
            .enumerate()
            .map(|(idx,_)| {
                CaseContextStruct::new(
                    &self.config,
                    &self.data[idx]
                )
            })
            .collect();
    }

}

pub type TaskResult = std::result::Result<Vec<CaseResult>, ()>;
