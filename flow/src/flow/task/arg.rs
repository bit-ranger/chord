use common::flow::Flow;

use crate::flow::case::arg::CaseArgStruct;
use common::value::{Json};
use common::error::Error;

#[derive(Debug)]
pub struct TaskArgStruct {
    data: Vec<Json>,
    flow: Flow,
    id: String
}


impl TaskArgStruct {

    pub const EMPTY_VEC: &'static Vec<(String, Json)> = &Vec::new();
    pub const EMPTY_DATA: &'static Json = &Json::Null;

    pub fn new(flow: Flow, data: Vec<Json>, id: &str) -> TaskArgStruct {
        let context = TaskArgStruct {
            flow,
            data,
            id: String::from(id)
        };
        return context;
    }


    pub fn data_case_vec<'p>(self: &TaskArgStruct, case_ctx: &'p Vec<(String, Json)>) -> Result<Vec<CaseArgStruct<'_, '_,'p>>, Error> {
        let case_point_id_vec = self.flow.case_point_id_vec()?;
        let vec = self.data.iter()
            .enumerate()
            .map(|(idx,_)| {
                CaseArgStruct::new(
                    idx,
                    &self.flow,
                    &self.data[idx],
                    case_point_id_vec.clone(),
                    case_ctx
                )
            })
            .collect();
        return Ok(vec);
    }

    pub fn pre_case(self: &TaskArgStruct) -> Option<CaseArgStruct<'_, '_, '_>> {
        let pre_pt_id_vec = self.pre_point_id_vec();
        return if pre_pt_id_vec.is_empty() {
            None
        } else {
            Some(
                CaseArgStruct::new(
                    0,
                    &self.flow,
                    TaskArgStruct::EMPTY_DATA,
                    pre_pt_id_vec,
                    TaskArgStruct::EMPTY_VEC,
                )
            )
        }

    }

    fn pre_point_id_vec(self: &TaskArgStruct) -> Vec<String> {
        let task_pt_chain_arr = self.flow.data()["task"]["pre"]["chain"].as_array();
        if task_pt_chain_arr.is_none() {
            return vec![];
        }
        let task_pt_chain_arr = task_pt_chain_arr.unwrap();
        let task_pt_chain_vec: Vec<String> = task_pt_chain_arr.iter()
            .map(|e| {
                e.as_str().map(|s| String::from(s)).unwrap()
            })
            .collect();

        return task_pt_chain_vec;
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

