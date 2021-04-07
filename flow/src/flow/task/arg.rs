use chord_common::flow::Flow;

use crate::flow::case::arg::CaseArgStruct;
use chord_common::value::{Json};
use chord_common::error::Error;
use async_std::sync::Arc;

#[derive(Debug)]
pub struct TaskArgStruct {
    data: Arc<Vec<Json>>,
    flow: Arc<Flow>,
    id: String
}


impl TaskArgStruct {

    pub fn new(flow: Flow, data: Vec<Json>, id: &str) -> TaskArgStruct {
        let context = TaskArgStruct {
            flow: Arc::new(flow),
            data: Arc::new(data),
            id: String::from(id)
        };
        return context;
    }


    pub fn case_arg_vec<'p>(self: &TaskArgStruct, case_ctx: Vec<(String, Json)>) -> Result<Vec<CaseArgStruct>, Error> {
        let case_ctx = Arc::new(case_ctx);
        let case_point_id_vec = self.flow.case_point_id_vec()?;
        let vec = self.data.iter()
            .enumerate()
            .map(|(idx,_)| {
                CaseArgStruct::new(
                    idx,
                    self.flow.clone(),
                    self.data.clone(),
                    case_point_id_vec.clone(),
                    case_ctx.clone()
                )
            })
            .collect();
        return Ok(vec);
    }

    pub fn pre_arg(self: &TaskArgStruct) -> Option<CaseArgStruct> {
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
                    self.flow.clone(),
                    Arc::new(Vec::new()),
                    pre_pt_id_vec,
                    Arc::new(Vec::new())
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

