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

    pub fn get_rate_limit(self: &TaskContextStruct) -> (usize, usize) {
        let num = match self.config["task"]["rate-limit"]["num"].as_u64() {
            Some(n) => n as usize,
            None => 10
        };

        let sec = match self.config["task"]["rate-limit"]["sec"].as_u64() {
            Some(n) => n as usize,
            None => 1
        };

        return (num,sec);
    }
}


