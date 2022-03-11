use std::sync::Arc;

use chrono::Utc;
use log::{error, info, trace, warn};

use chord_core::action::{Action, RunArg, Scope};
use chord_core::collection::TailDropVec;
use chord_core::flow::Flow;
use chord_core::step::StepState;
use chord_core::value::Value;
use res::StepAssessStruct;
use Error::*;

use crate::flow::step::arg::{CreateArgStruct, RunArgStruct};
use crate::flow::step::res::ActionAssessStruct;
use crate::model::app::FlowApp;
use crate::TaskIdSimple;

pub mod arg;
pub mod res;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("step `{0}` create:\n{1}")]
    Create(String, Box<dyn std::error::Error + Sync + Send>),
}

pub struct StepRunner {
    action_vec: Arc<TailDropVec<(String, Box<dyn Action>)>>,
}

impl StepRunner {
    pub async fn new(
        app: &dyn FlowApp,
        flow: &Flow,
        task_id: Arc<TaskIdSimple>,
        step_id: String,
    ) -> Result<StepRunner, Error> {
        let obj = flow.step_obj(step_id.as_str());

        let mut action_vec = Vec::with_capacity(obj.len());

        for (aid, _) in obj.iter() {
            let func = flow.step_action_func(step_id.as_str(), aid);

            let create_arg = CreateArgStruct::new(
                app,
                flow,
                None,
                task_id.clone(),
                func.into(),
                step_id.clone(),
                aid,
            );

            let action = app
                .get_action_factory(func.into())
                .unwrap()
                .create(&create_arg)
                .await
                .map_err(|e| Create(step_id.clone(), e))?;
            action_vec.push((aid.to_string(), action));
        }

        Ok(StepRunner {
            action_vec: Arc::new(TailDropVec::from(action_vec)),
        })
    }

    pub async fn run(&self, arg: &mut RunArgStruct<'_, '_>) -> StepAssessStruct {
        trace!("step start {}", arg.id());
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
                    arg.context().insert(key.to_string(), v.as_value().clone());
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
            info!("step Ok    {}", arg.id());
        } else {
            for ass in assess_vec.iter() {
                if let StepState::Ok(v) = ass.state() {
                    warn!("{} : {}  <<<  {}", ass.id(), v.as_value(), ass.explain());
                } else if let StepState::Err(e) = ass.state() {
                    error!("{} : {}  <<<  {}", ass.id(), e, ass.explain());
                }
            }
            error!("step Err   {}", arg.id());
        }

        StepAssessStruct::new(arg.id().clone(), start, Utc::now(), assess_vec)
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
