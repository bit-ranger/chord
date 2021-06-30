use std::borrow::Borrow;
use std::collections::HashSet;
use std::time::Duration;

use lazy_static::lazy_static;
use regex::Regex;

use crate::error::Error;
use crate::value::{Map, Value};
use crate::{err, rerr};

lazy_static! {
    pub static ref ID_PATTERN: Regex = Regex::new(r"^[\w]+$").unwrap();
}

#[derive(Debug, Clone)]
pub struct Flow {
    flow: Value,
}

impl Flow {
    pub fn new(flow: Value) -> Result<Flow, Error> {
        let flow = Flow { flow };

        flow._version()?;

        let mut step_id_checked: HashSet<&str> = HashSet::new();
        let pre_sid_vec = flow.pre_step_id_vec().unwrap_or(vec![]);
        for pre_sid in pre_sid_vec {
            if !ID_PATTERN.is_match(pre_sid) {
                return rerr!("step", format!("invalid step_id {}", pre_sid));
            }
            if step_id_checked.contains(pre_sid) {
                return rerr!("step", format!("duplicate step_id {}", pre_sid));
            } else {
                step_id_checked.insert(pre_sid.into());
            }
        }

        let stage_id_vec = flow._stage_id_vec()?;

        for stage_id in stage_id_vec {
            if !ID_PATTERN.is_match(stage_id) {
                return rerr!("stage", format!("invalid stage_id {}", stage_id));
            }

            flow._stage_concurrency(stage_id)?;
            flow._stage_duration(stage_id)?;
            flow._stage_round(stage_id)?;
            flow._stage_break_on(stage_id)?;

            let stage_sid_vec = flow._stage_step_id_vec(stage_id)?;
            for stage_sid in stage_sid_vec {
                if !ID_PATTERN.is_match(stage_sid) {
                    return rerr!("step", format!("invalid step_id {}", stage_sid));
                }

                if step_id_checked.contains(stage_sid) {
                    return rerr!("step", format!("duplicate step_id {}", stage_sid));
                } else {
                    step_id_checked.insert(stage_sid.into());
                }
            }
        }

        for sid in step_id_checked.iter() {
            flow.step(sid)
                .as_object()
                .ok_or_else(|| err!("step", format!("invalid step {}", sid)))?;

            let _ = flow._step_action(sid)?;
            let _ = flow._step_timeout(sid)?;
        }

        return Ok(flow);
    }

    pub fn version(&self) -> &str {
        self._version().unwrap()
    }

    pub fn stage_step_id_vec(&self, stage_id: &str) -> Vec<&str> {
        self._stage_step_id_vec(stage_id).unwrap()
    }

    pub fn pre_step_id_vec(&self) -> Option<Vec<&str>> {
        let task_step_chain_arr = self.flow["pre"]["step"]
            .as_object()
            .map(|p| p.keys().map(|k| k.as_str()).collect());
        if task_step_chain_arr.is_none() {
            return None;
        }
        return Some(task_step_chain_arr.unwrap());
    }

    pub fn step(&self, step_id: &str) -> &Value {
        for stage_id in self.stage_id_vec() {
            let step = self.flow["stage"][stage_id]["step"][step_id].borrow();
            if !step.is_null() {
                return step;
            }
        }

        return self.flow["pre"]["step"][step_id].borrow();
    }

    pub fn step_args(&self, step_id: &str) -> &Value {
        self.step(step_id)["args"].borrow()
    }

    pub fn def(&self) -> Option<&Map> {
        self.flow["def"].as_object()
    }

    pub fn step_action(&self, step_id: &str) -> &str {
        self._step_action(step_id).unwrap()
    }

    pub fn step_assert(&self, step_id: &str) -> Option<&str> {
        self.step(step_id)["assert"].as_str()
    }

    pub fn step_timeout(&self, step_id: &str) -> Duration {
        self._step_timeout(step_id).unwrap()
    }

    pub fn stage_id_vec(&self) -> Vec<&str> {
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

    pub fn stage_case_filter(&self, stage_id: &str) -> Option<&str> {
        self.flow["stage"][stage_id]["case"]["filter"].as_str()
    }

    pub fn stage_break_on(&self, stage_id: &str) -> &str {
        self._stage_break_on(stage_id).unwrap()
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

    fn _stage_step_id_vec(&self, stage_id: &str) -> Result<Vec<&str>, Error> {
        let sid_vec = self.flow["stage"][stage_id]["step"]
            .as_object()
            .map(|p| p.keys().map(|k| k.as_str()).collect())
            .ok_or(Error::new("stage", "missing stage step"))?;
        return Ok(sid_vec);
    }

    fn _step_action(&self, step_id: &str) -> Result<&str, Error> {
        self.step(step_id)["action"]
            .as_str()
            .ok_or(err!("step", "missing action"))
    }

    fn _step_timeout(&self, step_id: &str) -> Result<Duration, Error> {
        let s = self.step(step_id)["timeout"].as_u64();
        if s.is_none() {
            return Ok(Duration::from_secs(30));
        }

        let s = s.unwrap();
        if s < 1 {
            return rerr!("step", "timeout must > 0");
        }
        Ok(Duration::from_secs(s))
    }

    fn _stage_id_vec(&self) -> Result<Vec<&str>, Error> {
        let sid_vec = self.flow["stage"]
            .as_object()
            .map(|p| p.keys().map(|k| k.as_str()).collect())
            .ok_or(Error::new("stage", "missing stage"))?;
        return Ok(sid_vec);
    }

    fn _stage_concurrency(&self, stage_id: &str) -> Result<usize, Error> {
        let s = self.flow["stage"][stage_id]["concurrency"].as_u64();
        if s.is_none() {
            return Ok(10);
        }

        let s = s.unwrap();
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
            return Ok(Duration::from_secs(300));
        }

        let s = s.unwrap();
        if s < 1 {
            return rerr!("stage", "duration must > 0");
        }
        Ok(Duration::from_secs(s))
    }

    fn _stage_break_on(&self, stage_id: &str) -> Result<&str, Error> {
        let break_on = self.flow["stage"][stage_id]["break_on"]
            .as_str()
            .unwrap_or("never");
        match break_on {
            "never" => Ok(break_on),
            "stage_fail" => Ok(break_on),
            _ => rerr!("stage", "break_on unsupported value"),
        }
    }
}
