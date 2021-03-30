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


    pub fn case_arg_vec<'p>(self: &TaskArgStruct, case_ctx: &'p Vec<(String, Json)>) -> Result<Vec<CaseArgStruct<'_, '_,'p>>, Error> {
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

    pub fn pre_arg(self: &TaskArgStruct) -> Option<CaseArgStruct<'_, '_, '_>> {
        let pre_pt_id_vec = self.flow.pre_point_id_vec();
        if pre_pt_id_vec.is_none() {
           return None
        }
        let pre_pt_id_vec = pre_pt_id_vec.unwrap();
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



    pub fn limit_concurrency(self: &TaskArgStruct) -> usize {
        self.flow.limit_concurrency()
    }




    pub fn id(&self) -> &str {
        &self.id
    }
}

