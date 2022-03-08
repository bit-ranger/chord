use std::borrow::Borrow;
use std::collections::HashSet;
use std::path::Path;
use std::time::Duration;

use lazy_static::lazy_static;
use regex::Regex;

use crate::flow::Error::EntryLost;
use crate::flow::Error::*;
use crate::value::{Map, Value};

lazy_static! {
    pub static ref ID_PATTERN: Regex = Regex::new(r"^[\w]{1,50}$").unwrap();
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("invalid id `{0}`")]
    IdInvalid(String),
    #[error("duplicated id `{0}`")]
    IdDuplicated(String),
    #[error("{0} lost entry `{1}`")]
    EntryLost(String, String),
    #[error("{0} unexpect entry `{1}`")]
    EntryUnexpected(String, String),
    #[error("{0} must {1} but it {2}")]
    Violation(String, String, String),
    #[error("{0} unexpect value {1}")]
    ValueUnexpected(String, String),
}

#[derive(Debug, Clone)]
pub struct Flow {
    flow: Value,
    meta: Map,
}

#[derive(Debug, Clone)]
pub struct Then {
    cond: Option<String>,
    reg: Option<Map>,
    goto: Option<String>,
}

impl Then {
    pub fn cond(&self) -> Option<&str> {
        self.cond.as_ref().map(|s| s.as_str())
    }
    pub fn reg(&self) -> Option<&Map> {
        self.reg.as_ref()
    }
    pub fn goto(&self) -> Option<&str> {
        self.goto.as_ref().map(|g| g.as_str())
    }
}

impl Flow {
    pub fn new(flow: Value, dir: &Path) -> Result<Flow, Error> {
        let mut meta = Map::new();
        meta.insert(
            "task_dir".to_string(),
            Value::String(dir.to_path_buf().to_str().unwrap().to_string()),
        );

        let flow = Flow { flow, meta };

        flow._root_check()?;
        flow._version()?;

        let mut step_id_checked: HashSet<&str> = HashSet::new();
        let pre_step_id_vec = flow.pre_step_id_vec().unwrap_or(vec![]);
        for pre_step_id in pre_step_id_vec {
            if !ID_PATTERN.is_match(pre_step_id) {
                return Err(IdInvalid(pre_step_id.into()));
            }
            if step_id_checked.contains(pre_step_id) {
                return Err(IdDuplicated(pre_step_id.into()));
            } else {
                step_id_checked.insert(pre_step_id.into());
            }

            flow._pre_check()?;
        }

        let stage_id_vec = flow._stage_id_vec()?;
        if stage_id_vec.is_empty() {
            return Err(EntryLost("root".into(), "stage".into()));
        }

        for stage_id in stage_id_vec {
            if !ID_PATTERN.is_match(stage_id) {
                return Err(IdInvalid(stage_id.into()));
            }

            flow._stage_check(stage_id)?;
            flow._stage_concurrency(stage_id)?;
            flow._stage_duration(stage_id)?;
            flow._stage_round(stage_id)?;
            flow._stage_break_on(stage_id)?;

            let stage_step_id_vec = flow._stage_step_id_vec(stage_id)?;

            if stage_step_id_vec.is_empty() {
                return Err(EntryLost(stage_id.into(), "step".into()));
            }

            for stage_step_id in stage_step_id_vec {
                if !ID_PATTERN.is_match(stage_step_id) {
                    return Err(IdInvalid(stage_step_id.into()));
                }

                if step_id_checked.contains(stage_step_id) {
                    return Err(IdDuplicated(stage_step_id.into()));
                } else {
                    step_id_checked.insert(stage_step_id.into());
                }
            }
        }

        for step_id in step_id_checked.iter() {
            flow._step_check(step_id)?;
            flow._step_spec_check(step_id)?;

            let _ = flow._step_obj(step_id)?;
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

    pub fn meta(&self) -> &Map {
        &self.meta
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
        let id_vec: Vec<&str> = task_step_chain_arr.unwrap();
        return if id_vec.is_empty() {
            None
        } else {
            Some(id_vec)
        };
    }

    pub fn stage_id_vec(&self) -> Vec<&str> {
        self._stage_id_vec().unwrap()
    }

    pub fn stage_loader<'a, 's>(&'s self, stage_id: &'a str) -> &'a Value
    where
        's: 'a,
    {
        &self.flow["stage"][stage_id]["loader"]
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

    pub fn step_exec_args(&self, step_id: &str) -> &Value {
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

    pub fn step_then(&self, step_id: &str) -> Option<Vec<Then>> {
        self._step_then(step_id).unwrap()
    }

    pub fn step_obj(&self, step_id: &str) -> &Map {
        self._step_obj(step_id).unwrap()
    }

    // -----------------------------------------------
    // private

    fn _root_check(&self) -> Result<(), Error> {
        let enable_keys = vec!["version", "def", "stage", "pre"];
        let root = self.flow.borrow();
        let object = root
            .as_object()
            .ok_or_else(|| Violation("root".into(), "be a object".into(), "is not".into()))?;
        for (k, _) in object {
            if !enable_keys.contains(&k.as_str()) {
                return Err(EntryUnexpected("root".into(), k.into()));
            }
        }
        return Ok(());
    }

    fn _version(&self) -> Result<&str, Error> {
        let v = self.flow["version"]
            .as_str()
            .ok_or(EntryLost("root".into(), "version".into()))?;

        if v != "0.0.1" {
            return Err(Violation("version".into(), "0.0.1".into(), v.into()));
        } else {
            Ok(v)
        }
    }

    fn _pre_check(&self) -> Result<(), Error> {
        let enable_keys = vec!["step"];
        let pre = self.flow["pre"].borrow();
        let object = pre
            .as_object()
            .ok_or_else(|| Violation("pre".into(), "be a object".into(), "is not".into()))?;
        for (k, _) in object {
            if !enable_keys.contains(&k.as_str()) {
                return Err(EntryUnexpected("pre".into(), k.into()));
            }
        }
        return Ok(());
    }

    fn _stage_id_vec(&self) -> Result<Vec<&str>, Error> {
        let step_id_vec = self.flow["stage"]
            .as_object()
            .map(|p| p.keys().map(|k| k.as_str()).collect())
            .ok_or(EntryLost("root".into(), "stage".into()))?;
        return Ok(step_id_vec);
    }

    fn _stage_check(&self, stage_id: &str) -> Result<(), Error> {
        let enable_keys = vec![
            "step",
            "loader",
            "concurrency",
            "round",
            "duration",
            "break_on",
        ];
        let stage = self.flow["stage"][stage_id].borrow();
        let object = stage.as_object().ok_or_else(|| {
            Violation(
                format!("stage.{}", stage_id),
                "be a object".into(),
                "is not".into(),
            )
        })?;
        for (k, _) in object {
            if !enable_keys.contains(&k.as_str()) {
                return Err(EntryUnexpected(format!("stage.{}", stage_id), k.into()));
            }
        }
        return Ok(());
    }

    fn _stage_concurrency(&self, stage_id: &str) -> Result<usize, Error> {
        let s = self.flow["stage"][stage_id]["concurrency"].as_u64();
        if s.is_none() {
            return Ok(10);
        }

        let s = s.unwrap();
        if s < 1 {
            return Err(Violation(
                format!("stage.{}.concurrency", stage_id),
                "> 0".into(),
                format!("is {}", s),
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
            return Err(Violation(
                format!("stage.{}.round", stage_id),
                "> 0".into(),
                format!("is {}", s),
            ));
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
            return Err(Violation(
                format!("stage.{}.duration", stage_id),
                "> 0".into(),
                format!("is {}", s),
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
            o => Err(ValueUnexpected(
                format!("stage.{}.break_on", stage_id),
                o.into(),
            )),
        }
    }

    fn _stage_step_id_vec(&self, stage_id: &str) -> Result<Vec<&str>, Error> {
        let step_id_vec: Vec<&str> = self.flow["stage"][stage_id]["step"]
            .as_object()
            .map(|p| p.keys().map(|k| k.as_str()).collect())
            .ok_or(EntryLost(stage_id.into(), "step".into()))?;
        if step_id_vec.is_empty() {
            return Err(Violation(
                format!("stage.{}.step", stage_id),
                "not empty".into(),
                "is".into(),
            ));
        }
        return Ok(step_id_vec);
    }

    fn _step_check(&self, step_id: &str) -> Result<(), Error> {
        let step = self._step(step_id);
        let _ = step.as_object().ok_or_else(|| {
            Violation(
                format!("step.{}", step_id),
                "be a object".into(),
                "is not".into(),
            )
        })?;
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

    fn _step_obj(&self, step_id: &str) -> Result<&Map, Error> {
        let step = self._step(step_id);
        step.as_object().ok_or_else(|| {
            Violation(
                format!("step.{}", step_id),
                "be a object".into(),
                "is not".into(),
            )
        })
    }

    fn _step_let(&self, step_id: &str) -> Result<Option<&Map>, Error> {
        let lv = &self._step(step_id)["let"];
        if lv.is_null() {
            return Ok(None);
        }
        lv.as_object()
            .map(|o| Some(o))
            .ok_or(EntryLost(format!("step.{}", step_id), "let".into()))
    }

    fn _step_exec_action(&self, step_id: &str) -> Result<&str, Error> {
        let step = self._step(step_id).as_object().ok_or(Violation(
            step_id.into(),
            "be a object".into(),
            "is not".into(),
        ))?;

        let fixed_keys = vec!["let", "spec", "assert", "then"];
        let action: Vec<&str> = step
            .iter()
            .filter(|(k, _)| !fixed_keys.contains(&k.as_str()))
            .map(|(k, _)| k.as_str())
            .collect();

        if action.len() != 1 {
            return Err(Violation(
                format!("step.{}", step_id),
                "have only 1 action".into(),
                format!("has {}", action.len()),
            ));
        }

        Ok(action[0])
    }

    pub fn _step_exec_args(&self, step_id: &str) -> Result<&Value, Error> {
        let action = self._step_exec_action(step_id)?;
        let args = &self._step(step_id)[action];
        return if args.is_null() {
            Err(EntryLost(
                format!("step.{}.{}", step_id, action),
                "args".into(),
            ))
        } else {
            Ok(args)
        };
    }

    fn _step_spec_check(&self, step_id: &str) -> Result<(), Error> {
        let spec = self._step(step_id)["spec"].borrow();
        if spec.is_null() {
            return Ok(());
        } else {
            let enable_keys = vec!["timeout", "catch_err"];
            let object = spec.as_object().ok_or(Violation(
                format!("step.{}.spec", step_id),
                "be a object".into(),
                "is not".into(),
            ))?;

            for (k, _) in object {
                if !enable_keys.contains(&k.as_str()) {
                    return Err(EntryUnexpected(format!("step.{}", step_id), k.into()));
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
            return Err(Violation(
                format!("step.{}.spec.timeout", step_id),
                "> 0".into(),
                format!("is {}", s),
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
        match &self._step(step_id)["assert"] {
            Value::Null => Ok(None),
            Value::String(s) => Ok(Some(s.as_str())),
            _ => {
                return Err(Violation(
                    format!("step.{}.assert", step_id),
                    "be a string".into(),
                    "is not".into(),
                ));
            }
        }
    }

    pub fn _step_then(&self, step_id: &str) -> Result<Option<Vec<Then>>, Error> {
        let array = self._step(step_id)["then"].as_array();
        if array.is_none() {
            return Ok(None);
        } else {
            let array = array.unwrap();
            Ok(Some(
                array
                    .iter()
                    .filter(|v| v.is_object())
                    .map(|m| Then {
                        cond: m["if"].as_str().map(|c| c.to_string()),
                        reg: m["reg"].as_object().map(|r| r.clone()),
                        goto: m["goto"].as_str().map(|g| g.to_string()),
                    })
                    .collect(),
            ))
        }
    }
}
