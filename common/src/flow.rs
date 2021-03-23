use crate::value::Json;
use std::borrow::Borrow;
use crate::error::Error;

#[derive(Debug, Clone)]
pub struct Flow {
    flow: Json
}

impl Flow{

    pub fn new(flow: Json) -> Result<Flow,Error>{
        let flow = Flow{
            flow
        };
        flow.case_point_id_vec()?;

        return Ok(flow);
    }

    pub fn data(self: &Flow) -> &Json {
        self.flow.borrow()
    }

    pub fn case_point_id_vec(self: &Flow) -> Result<Vec<String>, Error> {
        let task_pt_chain_arr = self.flow["case"]["chain"].as_array()
            .ok_or(Error::new("flow", "missing case.chain"))?;
        let task_pt_chain_vec: Vec<String> = task_pt_chain_arr.iter()
            .map(|e| {
                e.as_str().map(|s| String::from(s)).unwrap()
            })
            .collect();

        return Ok(task_pt_chain_vec);
    }
}
