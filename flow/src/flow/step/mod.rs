use std::sync::Arc;

use chrono::Utc;
use log::{error, info, trace, warn, debug};

use chord_core::action::{Action, Arg, Id, Scope};
use chord_core::collection::TailDropVec;
use chord_core::step::StepState;
use chord_core::value::Value;
use res::StepAssessStruct;
use Error::*;

use crate::flow::step::arg::ArgStruct;
use crate::flow::step::res::ActionAssessStruct;

pub mod arg;
pub mod res;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("unsupported action `{0}`")]
    Unsupported(String),

    #[error("action `{0}.{1}` create:\n{1}")]
    Create(String, String, Box<dyn std::error::Error + Sync + Send>),
}

pub struct StepRunner {
    action_vec: Arc<TailDropVec<(String, Box<dyn Action>)>>,
}

impl StepRunner {
    pub async fn new(arg: &mut ArgStruct<'_, '_>) -> Result<StepRunner, Error> {
        trace!("step new {}", arg.id());
        let obj = arg.flow().step_obj(arg.id().step());
        let aid_vec: Vec<String> = obj.iter().map(|(aid, _)| aid.to_string()).collect();
        let mut action_vec = Vec::with_capacity(obj.len());

        for aid in aid_vec {
            arg.aid(aid.as_str());
            let func = arg.flow().step_action_func(arg.id().step(), aid.as_str());
            let action = arg
                .combo()
                .action(func.into())
                .ok_or_else(|| Unsupported(func.into()))?
                .action(arg)
                .await
                .map_err(|e| Create(arg.id().step().to_string(), aid.to_string(), e))?;
            action_vec.push((aid.to_string(), action));
        }

        Ok(StepRunner {
            action_vec: Arc::new(TailDropVec::from(action_vec)),
        })
    }

    pub async fn run(&self, arg: &mut ArgStruct<'_, '_>) -> StepAssessStruct {
        trace!("step run {}", arg.id());
        let start = Utc::now();
        let mut assess_vec = Vec::with_capacity(self.action_vec.len());
        let mut success = true;
        for (aid, action) in self.action_vec.iter() {
            let key: &str = aid;
            let action: &Box<dyn Action> = action;
            arg.aid(key);
            let explain = action.explain(arg).await.unwrap_or(Value::Null);
            let value = action.run(arg).await;
            match &value {
                Ok(v) => {
                    arg.context_mut()
                        .data_mut()
                        .insert(key.to_string(), v.as_value().clone());
                    let assess = action_assess_create(aid, explain, value);
                    assess_vec.push(assess);
                }

                Err(_) => {
                    let assess = action_assess_create(aid, explain, value);
                    assess_vec.push(assess);
                    success = false;
                    break;
                }
            }
        }

        if success {
            for ass in assess_vec.iter() {
                if let StepState::Ok(v) = ass.state() {
                    debug!(
                        "{}:\n{}\n>>> {}",
                        ass.id(),
                        explain_string(ass.explain()),
                        v.as_value()
                    );
                }
            }
            info!("step Ok   {}", arg.id());
        } else {
            for ass in assess_vec.iter() {
                if let StepState::Ok(v) = ass.state() {
                    warn!(
                        "{}:\n{}\n>>> {}",
                        ass.id(),
                        explain_string(ass.explain()),
                        v.as_value(),
                    );
                } else if let StepState::Err(e) = ass.state() {
                    error!(
                        "{}:\n{}\n>>> {}",
                        ass.id(),
                        explain_string(ass.explain()),
                        e
                    );
                }
            }
            error!("step Err {}", arg.id());
        }

        StepAssessStruct::new(Clone::clone(arg.id()), start, Utc::now(), assess_vec)
    }
}

fn action_assess_create(
    aid: &str,
    explain: Value,
    value: Result<Box<dyn Scope>, chord_core::action::Error>,
) -> ActionAssessStruct {
    return if let Err(_) = value.as_ref() {
        ActionAssessStruct::new(
            aid.to_string(),
            explain,
            StepState::Err(value.err().unwrap()),
        )
    } else {
        ActionAssessStruct::new(aid.to_string(), explain, StepState::Ok(value.unwrap()))
    };
}

fn explain_string(exp: &Value) -> String {
    if let Value::String(txt) = exp {
        txt.to_string()
    } else {
        exp.to_string()
    }
}
