use std::collections::BTreeMap;

use common::flow::Flow;

use crate::flow::case::arg::CaseArgStruct;

#[derive(Debug)]
pub struct TaskArgStruct {
    data: Vec<BTreeMap<String,String>>,
    flow: Flow,
    id: String
}


impl TaskArgStruct {

    pub fn new(flow: Flow, data: Vec<BTreeMap<String,String>>, id: &str) -> TaskArgStruct {
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

    pub fn limit_concurrency(self: &TaskArgStruct) -> usize {
        let num = match self.flow.data()["task"]["limit"]["concurrency"].as_u64() {
            Some(n) => n as usize,
            None => 9999
        };

        return num;
    }




    pub fn id(&self) -> &str {
        &self.id
    }
}

