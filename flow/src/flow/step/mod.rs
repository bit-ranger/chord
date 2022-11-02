use std::error::Error as StdError;
use std::sync::Arc;

use chrono::Utc;
use log::{debug, error, info, trace, warn};

use chord_core::action::{Action, Asset, Chord, Id};
use chord_core::collection::TailDropVec;
use chord_core::value::Value;
use Error::*;
use res::StepAssessStruct;

use crate::flow::step::arg::{ArgStruct, ChordStruct};
use crate::flow::step::res::{ActionAssessStruct, ActionState};

pub mod arg;
pub mod res;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("unsupported action `{0}`")]
    Unsupported(String),

    #[error("action `{0}.{1}` create:\n{1}")]
    Create(String, String, Box<dyn StdError + Sync + Send>),
}

pub struct StepRunner {
    chord: Arc<ChordStruct>,
    action_vec: Arc<TailDropVec<(String, Box<dyn Action>)>>,
}

impl StepRunner {
    pub async fn new(
        chord: Arc<ChordStruct>,
        arg: &mut ArgStruct<'_, '_>,
    ) -> Result<StepRunner, Error> {
        trace!("step new {}", arg.id());
        let obj = arg.flow().step_obj(arg.id().step());
        let aid_vec: Vec<String> = obj.iter().map(|(aid, _)| aid.to_string()).collect();
        let mut action_vec = Vec::with_capacity(obj.len());

        for aid in aid_vec {
            arg.aid(aid.as_str());
            let func = arg.flow().step_action_func(arg.id().step(), aid.as_str());
            let action = chord
                .creator(func.into())
                .ok_or_else(|| Unsupported(func.into()))?
                .create(chord.as_ref(), arg)
                .await
                .map_err(|e| Create(arg.id().step().to_string(), aid.to_string(), e))?;
            action_vec.push((aid.to_string(), action));
        }

        Ok(StepRunner {
            chord,
            action_vec: Arc::new(TailDropVec::from(action_vec)),
        })
    }

    pub async fn run(&self, arg: &mut ArgStruct<'_, '_>) -> StepAssessStruct {
        trace!("step run {}", arg.id());
        let start = Utc::now();
        let mut asset_vec = Vec::with_capacity(self.action_vec.len());
        let mut success = true;
        for (aid, action) in self.action_vec.iter() {
            let key: &str = aid;
            let action: &Box<dyn Action> = action;
            arg.aid(key);
            let explain = action
                .explain(self.chord.as_ref(), arg)
                .await
                .unwrap_or(Value::Null);
            let value = action.execute(self.chord.as_ref(), arg).await;
            match &value {
                Ok(v) => {
                    arg.context_mut()
                        .data_mut()
                        .insert(key.to_string(), v.to_value());
                    let assess = action_assess_flat(aid, explain, value);
                    asset_vec.extend(assess);
                }

                Err(_) => {
                    let assess = action_assess_flat(aid, explain, value);
                    asset_vec.extend(assess);
                    success = false;
                    break;
                }
            }
        }

        if success {
            for ass in asset_vec.iter() {
                if let ActionState::Ok(v) = ass.state() {
                    debug!(
                        "{}:\n{}\n>>> {}",
                        ass.id(),
                        explain_string(ass.explain()),
                        v.to_value()
                    );
                }
            }
            info!("step Ok   {}", arg.id());
        } else {
            for ass in asset_vec.iter() {
                if let ActionState::Ok(v) = ass.state() {
                    warn!(
                        "{}:\n{}\n>>> {}",
                        ass.id(),
                        explain_string(ass.explain()),
                        v.to_value()
                    );
                } else if let ActionState::Err(e) = ass.state() {
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

        StepAssessStruct::new(Clone::clone(arg.id()), start, Utc::now(), asset_vec)
    }
}

fn action_assess_flat(
    aid: &str,
    explain: Value,
    value: Result<Asset, chord_core::action::Error>,
) -> Vec<ActionAssessStruct> {
    return if let Err(_) = value.as_ref() {
        vec![ActionAssessStruct::new(
            aid.to_string(),
            explain,
            ActionState::Err(value.err().unwrap()),
        )]
    } else {
        match value.unwrap() {
            Asset::Value(v) => {
                vec![
                    ActionAssessStruct::new(aid.to_string(), explain, ActionState::Ok(Asset::Value(v)))
                ]
            }
            Asset::Data(d) => {
                vec![
                    ActionAssessStruct::new(aid.to_string(), explain, ActionState::Ok(Asset::Data(d)))
                ]
            }
            Asset::Frames(f) => {
                f.into_iter()
                    .map(|fi|
                        ActionAssessStruct::new(fi.id().to_string(),
                                                explain.clone(),
                                                ActionState::Ok(Asset::Frames(vec![fi]))))
                    .collect()
            }
        }
    };
}

fn explain_string(exp: &Value) -> String {
    if let Value::String(txt) = exp {
        txt.to_string()
    } else {
        exp.to_string()
    }
}

