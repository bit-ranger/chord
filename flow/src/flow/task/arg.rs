use std::collections::BTreeMap;

use common::value::Json;

use crate::flow::case::arg::CaseArgStruct;

#[derive(Debug)]
pub struct TaskArgStruct {
    data: Vec<BTreeMap<String,String>>,
    flow: Json,
    id: String
}


impl TaskArgStruct {

    pub fn new(flow: Json, data: Vec<BTreeMap<String,String>>, id: &str) -> TaskArgStruct {
        let context = TaskArgStruct {
            flow,
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
                    &self.flow,
                    &self.data[idx],
                    idx
                )
            })
            .collect();
    }

    pub fn get_limit_concurrency(self: &TaskArgStruct) -> usize {
        let num = match self.flow["task"]["limit"]["concurrency"].as_u64() {
            Some(n) => n as usize,
            None => 10
        };

        return num;
    }


    pub fn id(&self) -> &str {
        &self.id
    }
}


