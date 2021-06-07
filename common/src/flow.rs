use crate::error::Error;
use crate::step::POINT_ID_PATTERN;
use crate::value::{Json, Map};
use crate::{err, rerr};

use itertools::concat;
use std::borrow::Borrow;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct Flow {
    flow: Json,
}

impl Flow {
    pub fn new(flow: Json) -> Result<Flow, Error> {
        let flow = Flow { flow };

        flow._version()?;

        let stage_id_vec = flow._stage_id_vec()?;
        for stage_id in stage_id_vec.iter() {
            flow._stage_concurrency(stage_id)?;
            flow._stage_duration(stage_id)?;
            flow._stage_round(stage_id)?;
        }

        let case_sid_vec = flow._case_step_id_vec()?;
        for case_sid in case_sid_vec.iter() {
            if !POINT_ID_PATTERN.is_match(case_sid.as_str()) {
                return rerr!("step", format!("invalid step_id {}", case_sid));
            }
        }

        let pre_sid_vec = flow.pre_step_id_vec().unwrap_or(vec![]);
        for pre_sid in pre_sid_vec.iter() {
            if !POINT_ID_PATTERN.is_match(pre_sid.as_str()) {
                return rerr!("step", format!("invalid step_id {}", pre_sid));
            }

            if case_sid_vec.contains(pre_sid) {
                return rerr!("step", format!("duplicate step_id {}", pre_sid));
            }
        }

        for sid in concat(vec![case_sid_vec, pre_sid_vec]).iter() {
            flow.step(sid)
                .as_object()
                .ok_or_else(|| err!("step", format!("invalid step {}", sid)))?;

            let _ = flow._step_kind(sid)?;
            let _ = flow._step_timeout(sid)?;
        }

        return Ok(flow);
    }

    pub fn version(&self) -> &str {
        self._version().unwrap()
    }

    pub fn case_step_id_vec(&self) -> Vec<String> {
        self._case_step_id_vec().unwrap()
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

    pub fn def(&self) -> Option<&Map> {
        self.flow["def"].as_object()
    }

    pub fn step_kind(&self, step_id: &str) -> &str {
        self._step_kind(step_id).unwrap()
    }

    pub fn step_timeout(&self, step_id: &str) -> Duration {
        self._step_timeout(step_id).unwrap()
    }

    pub fn stage_id_vec(&self) -> Vec<String> {
        self._stage_id_vec().unwrap()
    }

    pub fn stage_concurrency(&self, stage_id: &str) -> usize {
        self._stage_concurrency(stage_id).unwrap()
    }

    pub fn stage_round(&self, stage_id: &str) -> usize {
        self._stage_round(stage_id).unwrap()
    }

    pub fn stage_duration(&self, stage_id: &str) -> Duration {
        self._stage_duration(stage_id).unwrap()
    }

    // -----------------------------------------------
    // private

    fn _version(&self) -> Result<&str, Error> {
        let v = self.flow["version"]
            .as_str()
            .ok_or(err!("version", "missing version"))?;

        if v != "0.0.1" {
            return rerr!("version", "version only supports 0.0.1");
        } else {
            Ok(v)
        }
    }

    fn _case_step_id_vec(&self) -> Result<Vec<String>, Error> {
        let sid_vec = self.flow["case"]["step"]
            .as_object()
            .map(|p| p.keys().map(|k| k.to_string()).collect())
            .ok_or(Error::new("case", "missing case.step"))?;
        return Ok(sid_vec);
    }

    fn _step_kind(&self, step_id: &str) -> Result<&str, Error> {
        self.step(step_id)["kind"]
            .as_str()
            .ok_or(err!("step", "missing kind"))
    }

    fn _step_timeout(&self, step_id: &str) -> Result<Duration, Error> {
        let s = self.step(step_id)["timeout"].as_u64();
        if s.is_none() {
            return Ok(Duration::from_secs(10));
        }

        let s = s.unwrap();
        if s < 1 {
            return rerr!("step", "timeout must > 0");
        }
        Ok(Duration::from_secs(10))
    }

    fn _stage_id_vec(&self) -> Result<Vec<String>, Error> {
        let sid_vec = self.flow["stage"]
            .as_object()
            .map(|p| p.keys().map(|k| k.to_string()).collect())
            .ok_or(Error::new("stage", "missing stage"))?;
        return Ok(sid_vec);
    }

    fn _stage_concurrency(&self, stage_id: &str) -> Result<usize, Error> {
        let s = self.flow["stage"][stage_id]["concurrency"].as_u64()
            .ok_or(err!("stage", "missing concurrency"))?;

        if s < 1 {
            return rerr!("stage", "concurrency must > 0");
        }
        Ok(s as usize)
    }

    fn _stage_round(&self, stage_id: &str) -> Result<usize, Error> {
        let s = self.flow["stage"][stage_id]["round"].as_u64();
        if s.is_none() {
            return Ok(1);
        }

        let s = s.unwrap();
        if s < 1 {
            return rerr!("stage", "round must > 0");
        }
        Ok(s as usize)
    }

    fn _stage_duration(&self, stage_id: &str) -> Result<Duration, Error> {
        let s = self.flow["stage"][stage_id]["duration"].as_u64();
        if s.is_none() {
            return Ok(Duration::from_secs(600));
        }

        let s = s.unwrap();
        if s < 1 {
            return rerr!("stage", "duration must > 0");
        }
        Ok(Duration::from_secs(s))
    }
}
