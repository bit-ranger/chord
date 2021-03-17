use std::collections::BTreeMap;

use crate::model::value::Json;
use crate::flow::case::arg::CaseArgStruct;


#[derive(Debug)]
pub struct TaskArgStruct {
    data: Vec<BTreeMap<String,String>>,
    config: Json,
    id: String
}


impl TaskArgStruct {

    pub fn new(config: Json, data: Vec<BTreeMap<String,String>>, id: &str) -> TaskArgStruct {
        let context = TaskArgStruct {
            config,
            data,
            id: String::from(id)
        };
        return context;
    }


    pub fn create_case(self: &TaskArgStruct) -> Vec<CaseArgStruct<'_, '_>> {
        return self.data.iter()
            .enumerate()
            .map(|(idx,_)| {
                CaseArgStruct::new(
                    &self.config,
                    &self.data[idx],
                    idx
                )
            })
            .collect();
    }

    pub fn get_limit_concurrency(self: &TaskArgStruct) -> usize {
        let num = match self.config["task"]["limit"]["concurrency"].as_u64() {
            Some(n) => n as usize,
            None => 10
        };

        return num;
    }


    pub fn id(&self) -> &str {
        &self.id
    }
}


