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
        let pre_step_id_vec = flow.pre_step_id_vec().unwrap_or(vec![]);
        for pre_step_id in pre_step_id_vec {
            if !ID_PATTERN.is_match(pre_step_id) {
                return Err(err!("flow", format!("step {} invalid id", pre_step_id)));
            }
            if step_id_checked.contains(pre_step_id) {
                return Err(err!("flow", format!("step {} duplicated id", pre_step_id)));
            } else {
                step_id_checked.insert(pre_step_id.into());
            }
        }

        let stage_id_vec = flow._stage_id_vec()?;
        if stage_id_vec.is_empty() {
            return Err(err!("flow", "stage missing"));
        }

        for stage_id in stage_id_vec {
            if !ID_PATTERN.is_match(stage_id) {
                return Err(err!("flow", format!("stage {} invalid id", stage_id)));
            }

            flow._stage_concurrency(stage_id)?;
            flow._stage_duration(stage_id)?;
            flow._stage_round(stage_id)?;
            flow._stage_break_on(stage_id)?;

            let stage_step_id_vec = flow._stage_step_id_vec(stage_id)?;

            if stage_step_id_vec.is_empty() {
                return Err(err!("flow", format!("stage {} missing step", stage_id)));
            }

            for stage_step_id in stage_step_id_vec {
                if !ID_PATTERN.is_match(stage_step_id) {
                    return Err(err!("flow", format!("stage {} invalid id", stage_step_id)));
                }

                if step_id_checked.contains(stage_step_id) {
                    return Err(err!(
                        "flow",
                        format!("step {} duplicated id", stage_step_id)
                    ));
                } else {
                    step_id_checked.insert(stage_step_id.into());
                }
            }
        }

        for step_id in step_id_checked.iter() {
            flow._step_check(step_id)?;
            flow._step_exec_check(step_id)?;
            flow._step_spec_check(step_id)?;

            let _ = flow._step_let(step_id)?;
            let _ = flow._step_assert(step_id)?;
            let _ = flow._step_then(step_id)?;
            let _ = flow._step_exec_action(step_id)?;
            let _ = flow._step_exec_args(step_id)?;
            let _ = flow._step_spec_timeout(step_id)?;
            let _ = flow._step_spec_catch_err(step_id)?;
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

    pub fn stage_case_name<'a, 's>(&'s self, stage_id: &'a str) -> &'a str
    where
        's: 'a,
    {
        self.flow["stage"][stage_id]["case"]["name"]
            .as_str()
            .unwrap_or(stage_id)
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

    pub fn stage_break_on(&self, stage_id: &str) -> &str {
        self._stage_break_on(stage_id).unwrap()
    }

    pub fn step_let(&self, step_id: &str) -> Option<&Map> {
        self._step_let(step_id).unwrap()
    }

    pub fn step_exec_action(&self, step_id: &str) -> &str {
        self._step_exec_action(step_id).unwrap()
    }

    pub fn step_exec_args(&self, step_id: &str) -> &Map {
        self._step_exec_args(step_id).unwrap()
    }

    pub fn step_spec_timeout(&self, step_id: &str) -> Duration {
        self._step_spec_timeout(step_id).unwrap()
    }

    pub fn step_spec_catch_err(&self, step_id: &str) -> bool {
        self._step_spec_catch_err(step_id).unwrap()
    }

    pub fn step_assert(&self, step_id: &str) -> Option<&str> {
        self._step_assert(step_id).unwrap()
    }

    pub fn step_then(&self, step_id: &str) -> Option<Vec<&Map>> {
        self._step_then(step_id).unwrap()
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
        let step_id_vec = self.flow["stage"][stage_id]["step"]
            .as_object()
            .map(|p| p.keys().map(|k| k.as_str()).collect())
            .ok_or(err!("flow", format!("stage {} missing step", stage_id)))?;
        return Ok(step_id_vec);
    }

    fn _step_check(&self, step_id: &str) -> Result<(), Error> {
        let enable_keys = vec!["let", "spec", "exec", "assert", "then"];
        let step = self._step(step_id);
        let object = step
            .as_object()
            .ok_or_else(|| err!("flow", format!("step {} must be a object", step_id)))?;
        for (k, _) in object {
            if !enable_keys.contains(&k.as_str()) {
                return Err(err!(
                    "flow",
                    format!("unexpected key {} in step.{}", k, step_id)
                ));
            }
        }
        return Ok(());
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

    pub fn _step_let(&self, step_id: &str) -> Result<Option<&Map>, Error> {
        let lv = &self._step(step_id)["let"];
        if lv.is_null() {
            return Ok(None);
        }
        lv.as_object()
            .map(|o| Some(o))
            .ok_or(err!("flow", format!("step {} missing let", step_id)))
    }

    fn _step_exec_check(&self, step_id: &str) -> Result<(), Error> {
        let enable_keys = vec!["action", "args"];

        let object = self._step(step_id)["exec"].as_object().ok_or(err!(
            "flow",
            format!("step {} exec must be a object", step_id)
        ))?;

        for (k, _) in object {
            if !enable_keys.contains(&k.as_str()) {
                return Err(err!(
                    "flow",
                    format!("unexpected key {} in step.{}.exec", k, step_id)
                ));
            }
        }

        Ok(())
    }

    fn _step_exec_action(&self, step_id: &str) -> Result<&str, Error> {
        self._step(step_id)["exec"]["action"].as_str().ok_or(err!(
            "flow",
            format!("step {} missing exec.action", step_id)
        ))
    }

    pub fn _step_exec_args(&self, step_id: &str) -> Result<&Map, Error> {
        self._step(step_id)["exec"]["args"]
            .as_object()
            .ok_or(err!("flow", format!("step {} missing exec.args", step_id)))
    }

    fn _step_spec_check(&self, step_id: &str) -> Result<(), Error> {
        let spec = self._step(step_id)["spec"].borrow();
        if spec.is_null() {
            return Ok(());
        } else {
            let enable_keys = vec!["timeout", "catch_err"];
            let object = spec.as_object().ok_or(err!(
                "flow",
                format!("step {} spec must be a object", step_id)
            ))?;

            for (k, _) in object {
                if !enable_keys.contains(&k.as_str()) {
                    return Err(err!(
                        "flow",
                        format!("unexpected key {} in step.{}.spec", k, step_id)
                    ));
                }
            }

            Ok(())
        }
    }

    fn _step_spec_timeout(&self, step_id: &str) -> Result<Duration, Error> {
        let s = self._step(step_id)["spec"]["timeout"].as_u64();
        if s.is_none() {
            return Ok(Duration::from_secs(60));
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

    fn _step_spec_catch_err(&self, step_id: &str) -> Result<bool, Error> {
        Ok(self._step(step_id)["spec"]["catch_err"]
            .as_bool()
            .unwrap_or(false))
    }

    pub fn _step_assert(&self, step_id: &str) -> Result<Option<&str>, Error> {
        Ok(self._step(step_id)["assert"].as_str())
    }

    pub fn _step_then(&self, step_id: &str) -> Result<Option<Vec<&Map>>, Error> {
        let array = self._step(step_id)["then"].as_array();
        if array.is_none() {
            return Ok(None);
        } else {
            let array = array.unwrap();
            Ok(Some(
                array
                    .iter()
                    .filter(|v| v.is_object())
                    .map(|m| m.as_object().unwrap())
                    .collect(),
            ))
        }
    }

    fn _stage_id_vec(&self) -> Result<Vec<&str>, Error> {
        let step_id_vec = self.flow["stage"]
            .as_object()
            .map(|p| p.keys().map(|k| k.as_str()).collect())
            .ok_or(err!("flow", "stage missing"))?;
        return Ok(step_id_vec);
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
            .unwrap_or("stage_fail");
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
