use crate::error::Error;
use crate::value::{Json, Map};
use crate::{err, rerr};
use crate::point::POINT_ID_PATTERN;


use std::borrow::Borrow;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct Flow {
    flow: Json,
}



impl Flow {
    pub fn new(flow: Json) -> Result<Flow, Error> {
        let flow = Flow { flow };
        let pt_id_vec = flow.case_point_id_vec()?;
        for pt_id in pt_id_vec {
            if !POINT_ID_PATTERN.is_match(pt_id.as_str()) {
                return rerr!("point", format!("invalid point_id {}", pt_id));
            }

            flow.point(pt_id.as_str())
                .as_object()
                .ok_or_else(|| err!("point", format!("invalid point_id {}", pt_id)))?;

            let _ = flow.point_kind(pt_id.as_str());
        }
        return Ok(flow);
    }

    pub fn case_point_id_vec(self: &Flow) -> Result<Vec<String>, Error> {
        let task_pt_chain_arr = self.flow["case"]["chain"]
            .as_array()
            .ok_or(Error::new("case", "missing case.chain"))?;
        return Ok(Flow::conv_to_string_vec(task_pt_chain_arr));
    }

    pub fn point(&self, point_id: &str) -> &Json {
        self.flow["point"][point_id].borrow()
    }

    pub fn point_config(&self, point_id: &str) -> &Json {
        self.flow["point"][point_id]["config"].borrow()
    }

    pub fn point_timeout(&self, point_id: &str) -> Duration {
        self.flow["point"][point_id]["timeout"]
            .as_u64()
            .map_or(Duration::from_secs(5), |sec| Duration::from_secs(sec))
    }

    pub fn point_kind(&self, point_id: &str) -> &str {
        self.flow["point"][point_id]["kind"].as_str().unwrap()
    }

    pub fn task_def(&self) -> Option<&Map> {
        self.flow["task"]["def"].as_object()
    }

    pub fn pre_point_id_vec(&self) -> Option<Vec<String>> {
        let task_pt_chain_arr = self.flow["task"]["pre"]["chain"].as_array();
        if task_pt_chain_arr.is_none() {
            return None;
        }
        return Some(Flow::conv_to_string_vec(task_pt_chain_arr.unwrap()));
    }

    pub fn limit_concurrency(&self) -> usize {
        let num = match self.flow["task"]["limit"]["concurrency"].as_u64() {
            Some(n) => n as usize,
            None => 9999,
        };

        return num;
    }

    fn conv_to_string_vec(vec: &Vec<Json>) -> Vec<String> {
        let string_vec: Vec<String> = vec
            .iter()
            .map(|e| e.as_str().map(|s| String::from(s)).unwrap())
            .collect();
        return string_vec;
    }
}
