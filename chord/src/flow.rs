use std::borrow::Borrow;
use std::collections::HashSet;
use std::time::Duration;

use lazy_static::lazy_static;
use regex::Regex;

use crate::err;
use crate::error::Error;
use crate::value::{Map, Value};

lazy_static! {
    pub static ref ID_PATTERN: Regex = Regex::new(r"^[\w]{1,50}$").unwrap();
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
                return Err(err!("flow", format!("step {} invalid id", pre_sid)));
            }
            if step_id_checked.contains(pre_sid) {
                return Err(err!("flow", format!("step {} duplicated id", pre_sid)));
            } else {
                step_id_checked.insert(pre_sid.into());
            }
        }

        let stage_id_vec = flow._stage_id_vec()?;

        for stage_id in stage_id_vec {
            if !ID_PATTERN.is_match(stage_id) {
                return Err(err!("flow", format!("stage {} invalid id", stage_id)));
            }

            flow._stage_concurrency(stage_id)?;
            flow._stage_duration(stage_id)?;
            flow._stage_round(stage_id)?;
            flow._stage_break_on(stage_id)?;

            let stage_sid_vec = flow._stage_step_id_vec(stage_id)?;
            for stage_sid in stage_sid_vec {
                if !ID_PATTERN.is_match(stage_sid) {
                    return Err(err!("flow", format!("stage {} invalid id", stage_sid)));
                }

                if step_id_checked.contains(stage_sid) {
                    return Err(err!("flow", format!("step {} duplicated id", stage_sid)));
                } else {
                    step_id_checked.insert(stage_sid.into());
                }
            }
        }

        for sid in step_id_checked.iter() {
            flow._step(sid)
                .as_object()
                .ok_or_else(|| err!("flow", format!("step {} invalid content", sid)))?;

            let _ = flow._step_exec_action(sid)?;
            let _ = flow._step_spec_timeout(sid)?;
        }

        return Ok(flow);
    }

    pub fn version(&self) -> &str {
        self._version().unwrap()
    }

    pub fn def(&self) -> Option<&Map> {
        self.flow["def"].as_object()
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

    pub fn stage_case_name(&self, stage_id: &str) -> &str {
        self.flow["stage"][stage_id]["case"]["name"]
            .as_str()
            .unwrap_or("case")
    }

    pub fn stage_break_on(&self, stage_id: &str) -> &str {
        self._stage_break_on(stage_id).unwrap()
    }

    pub fn step_let(&self, step_id: &str) -> &Value {
        self._step(step_id)["let"].borrow()
    }

    pub fn step_exec_action(&self, step_id: &str) -> &str {
        self._step_exec_action(step_id).unwrap()
    }

    pub fn step_exec_args(&self, step_id: &str) -> &Value {
        self._step(step_id)["exec"]["args"].borrow()
    }

    pub fn step_spec_timeout(&self, step_id: &str) -> Duration {
        self._step_spec_timeout(step_id).unwrap()
    }

    pub fn step_spec_catch_err(&self, step_id: &str) -> bool {
        self._step(step_id)["spec"]["catch_err"]
            .as_bool()
            .unwrap_or(false)
    }

    pub fn step_assert(&self, step_id: &str) -> Option<&str> {
        self._step(step_id)["assert"].as_str()
    }

    pub fn step_then(&self, step_id: &str) -> Option<Vec<&Map>> {
        let array = self._step(step_id)["then"].as_array()?;
        Some(
            array
                .iter()
                .filter(|v| v.is_object())
                .map(|m| m.as_object().unwrap())
                .collect(),
        )
    }

    // -----------------------------------------------
    // private

    fn _version(&self) -> Result<&str, Error> {
        let v = self.flow["version"]
            .as_str()
            .ok_or(err!("flow", "version missing"))?;

        if v != "0.0.1" {
            return Err(err!("flow", format!("version {} not supported", v)));
        } else {
            Ok(v)
        }
    }

    fn _stage_step_id_vec(&self, stage_id: &str) -> Result<Vec<&str>, Error> {
        let sid_vec = self.flow["stage"][stage_id]["step"]
            .as_object()
            .map(|p| p.keys().map(|k| k.as_str()).collect())
            .ok_or(err!("flow", format!("stage {} missing step", stage_id)))?;
        return Ok(sid_vec);
    }

    fn _step(&self, step_id: &str) -> &Value {
        for stage_id in self.stage_id_vec() {
            let step = self.flow["stage"][stage_id]["step"][step_id].borrow();
            if !step.is_null() {
                return step;
            }
        }

        return self.flow["pre"]["step"][step_id].borrow();
    }

    fn _step_exec_action(&self, step_id: &str) -> Result<&str, Error> {
        self._step(step_id)["exec"]["action"].as_str().ok_or(err!(
            "flow",
            format!("step {} missing exec.action", step_id)
        ))
    }

    fn _step_spec_timeout(&self, step_id: &str) -> Result<Duration, Error> {
        let s = self._step(step_id)["spec"]["timeout"].as_u64();
        if s.is_none() {
            return Ok(Duration::from_secs(10));
        }

        let s = s.unwrap();
        if s < 1 {
            return Err(err!(
                "flow",
                format!("step {} spec.timeout must > 0", step_id)
            ));
        }
        Ok(Duration::from_secs(s))
    }

    fn _stage_id_vec(&self) -> Result<Vec<&str>, Error> {
        let sid_vec = self.flow["stage"]
            .as_object()
            .map(|p| p.keys().map(|k| k.as_str()).collect())
            .ok_or(err!("flow", "stage missing"))?;
        return Ok(sid_vec);
    }

    fn _stage_concurrency(&self, stage_id: &str) -> Result<usize, Error> {
        let s = self.flow["stage"][stage_id]["concurrency"].as_u64();
        if s.is_none() {
            return Ok(10);
        }

        let s = s.unwrap();
        if s < 1 {
            return Err(err!(
                "flow",
                format!("stage {} concurrency must > 0", stage_id)
            ));
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
            return Err(err!("flow", format!("stage {} round must > 0", stage_id)));
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
            return Err(err!(
                "flow",
                format!("stage {} duration must > 0", stage_id)
            ));
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
            o => Err(err!(
                "flow",
                format!("stage {} break_on {} unsupported", stage_id, o)
            )),
        }
    }
}
