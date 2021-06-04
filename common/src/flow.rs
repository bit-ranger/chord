use crate::error::Error;
use crate::step::POINT_ID_PATTERN;
use crate::value::{Json, Map};
use crate::{err, rerr};

use std::borrow::Borrow;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct Flow {
    flow: Json,
}

impl Flow {
    pub fn new(flow: Json) -> Result<Flow, Error> {
        let flow = Flow { flow };

        flow.version()?;
        let case_sid_vec = flow.case_step_id_vec()?;
        for case_sid in case_sid_vec.iter() {
            if !POINT_ID_PATTERN.is_match(case_sid.as_str()) {
                return rerr!("step", format!("invalid step_id {}", case_sid));
            }

            flow.step(case_sid.as_str())
                .as_object()
                .ok_or_else(|| err!("step", format!("invalid step_id {}", case_sid)))?;

            let _ = flow.step_kind(case_sid.as_str());
        }

        let pre_sid_vec = flow.pre_step_id_vec().unwrap_or(vec![]);
        for pre_sid in pre_sid_vec.iter() {
            if !POINT_ID_PATTERN.is_match(pre_sid.as_str()) {
                return rerr!("step", format!("invalid step_id {}", pre_sid));
            }

            if case_sid_vec.contains(pre_sid) {
                return rerr!("step", format!("duplicate step_id {}", pre_sid));
            }

            flow.step(pre_sid.as_str())
                .as_object()
                .ok_or_else(|| err!("step", format!("invalid step_id {}", pre_sid)))?;

            let _ = flow.step_kind(pre_sid.as_str());
        }

        return Ok(flow);
    }

    pub fn version(self: &Flow) -> Result<&str, Error> {
        let v = self.flow["version"]
            .as_str()
            .ok_or(err!("version", "missing version"))?;

        if v != "0.0.1"{
            rerr!("version", "unsupported version")
        } else {
            Ok(v)
        }
    }

    pub fn case_step_id_vec(self: &Flow) -> Result<Vec<String>, Error> {
        let sid_vec = self.flow["case"]["step"]
            .as_object()
            .map(|p| p.keys().map(|k| k.to_string()).collect())
            .ok_or(Error::new("case", "missing case.step"))?;
        return Ok(sid_vec);
    }

    pub fn pre_step_id_vec(&self) -> Option<Vec<String>> {
        let task_step_chain_arr = self.flow["pre"]["step"]
            .as_object()
            .map(|p| p.keys().map(|k| k.to_string()).collect());
        if task_step_chain_arr.is_none() {
            return None;
        }
        return Some(task_step_chain_arr.unwrap());
    }

    pub fn step(&self, step_id: &str) -> &Json {
        let case_step = self.flow["case"]["step"][step_id].borrow();
        if !case_step.is_null() {
            case_step
        } else {
            self.flow["pre"]["step"][step_id].borrow()
        }
    }

    pub fn step_config(&self, step_id: &str) -> &Json {
        self.step(step_id)["config"].borrow()
    }

    pub fn step_timeout(&self, step_id: &str) -> Duration {
        self.step(step_id)["timeout"]
            .as_u64()
            .map_or(Duration::from_secs(5), |sec| Duration::from_secs(sec))
    }

    pub fn step_kind(&self, step_id: &str) -> &str {
        self.step(step_id)["kind"].as_str().unwrap()
    }

    pub fn def(&self) -> Option<&Map> {
        self.flow["def"].as_object()
    }

    pub fn ctrl_concurrency(&self) -> usize {
        let num = match self.flow["ctrl"]["concurrency"].as_u64() {
            Some(n) => n as usize,
            None => 100,
        };

        return num;
    }
}
