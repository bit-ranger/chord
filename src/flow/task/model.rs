use std::collections::BTreeMap;

use crate::model::value::Json;
use crate::flow::case::model::CaseContextStruct;


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


