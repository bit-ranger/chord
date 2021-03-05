use std::collections::BTreeMap;

use crate::model::Json;
use crate::model::Error;

use crate::model::case::{CaseContextStruct, CaseResult};

#[derive(Debug)]
pub struct TaskContextStruct {
    data: Vec<BTreeMap<String,String>>,
    config: Json
}


impl TaskContextStruct {

    pub fn new(config: Json, data: Vec<BTreeMap<String,String>>) -> TaskContextStruct {
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

pub type TaskResult = std::result::Result<Vec<CaseResult>, Error>;
