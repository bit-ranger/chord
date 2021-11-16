use chrono::{DateTime, Utc};
use log::{error, info, trace, warn};

use chord::case::CaseState;
use chord::collection::TailDropVec;
use chord::step::{StepAssess, StepState};
use res::CaseAssessStruct;

use crate::flow::case::arg::CaseArgStruct;
use crate::flow::step;
use crate::flow::step::arg::RunIdStruct;
use crate::flow::step::res::StepAssessStruct;
use crate::model::app::FlowApp;
use chord::action::Action;
use chord::value::Value;
use handlebars::TemplateRenderError;
use Error::*;

pub mod arg;
pub mod res;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("unrecognized step_id: {0}")]
    StepId(String),

    #[error("{0} render error: {1}")]
    Render(String, TemplateRenderError),
}

pub async fn run(flow_ctx: &dyn FlowApp, mut arg: CaseArgStruct) -> CaseAssessStruct {
    trace!("case start {}", arg.id());
    let start = Utc::now();
    let mut step_assess_vec = Vec::<Box<dyn StepAssess>>::new();
    let step_vec = arg.step_vec().clone();
    let mut step_id = step_vec[0].0.clone();
    loop {
        let action = get_action_by_step_id(step_vec.as_ref(), step_id.as_str());
        if action.is_none() {
            return case_fail_by_step_err(
                step_id.as_str(),
                arg,
                StepId(step_id.to_string()),
                step_assess_vec,
                start,
            );
        }
        let action = action.unwrap();

        let step_arg = arg.step_arg_create(step_id.as_str(), flow_ctx);
        if let Err(e) = step_arg {
            return case_fail_by_step_err(step_id.as_str(), arg, e, step_assess_vec, start);
        }
        let mut step_arg = step_arg.unwrap();

        let step_assess = step::run(flow_ctx, &mut step_arg, action).await;

        if !step_assess.state().is_ok() {
            step_assess_vec.push(Box::new(step_assess));
            warn!("case Fail  {}", arg.id());
            return CaseAssessStruct::new(
                arg.id().clone(),
                start,
                Utc::now(),
                arg.take_data(),
                CaseState::Fail(TailDropVec::from(step_assess_vec)),
            );
        } else {
            arg.step_ok_register(step_assess.id().step(), &step_assess)
                .await;

            let step_then = step_assess.then();
            if let Some(step_then) = step_then {
                if let Some(goto) = step_then.goto() {
                    step_id = goto.to_string();
                    step_assess_vec.push(Box::new(step_assess));
                    continue;
                }
            }

            step_assess_vec.push(Box::new(step_assess));
            let next = next_step_id(step_vec.as_ref(), step_id.as_str());
            if next.is_none() {
                break;
            } else {
                step_id = next.unwrap().to_string()
            }
        }
    }

    info!("case Ok   {}", arg.id());
    return CaseAssessStruct::new(
        arg.id().clone(),
        start,
        Utc::now(),
        arg.take_data(),
        CaseState::Ok(TailDropVec::from(step_assess_vec)),
    );
}

fn case_fail_by_step_err(
    step_id: &str,
    arg: CaseArgStruct,
    e: Error,
    mut step_assess_vec: Vec<Box<dyn StepAssess>>,
    start: DateTime<Utc>,
) -> CaseAssessStruct {
    error!("step Err {}\n{}", step_id, e);

    let step_run_id = RunIdStruct::new(step_id.to_string(), arg.id());
    let step_assess = StepAssessStruct::new(
        step_run_id,
        Utc::now(),
        Utc::now(),
        Value::Null,
        StepState::Err(Box::new(e)),
        None,
    );
    step_assess_vec.push(Box::new(step_assess));
    warn!("case Fail {}", arg.id());
    return CaseAssessStruct::new(
        arg.id().clone(),
        start,
        Utc::now(),
        arg.take_data(),
        CaseState::Fail(TailDropVec::from(step_assess_vec)),
    );
}

pub fn get_action_by_step_id<'v, 'a>(
    step_vec: &'v TailDropVec<(String, Box<dyn Action>)>,
    step_id: &str,
) -> Option<&'a dyn Action>
where
    'v: 'a,
{
    let step_idx: usize = get_idx_by_id(step_vec, step_id)?;
    step_vec.get(step_idx).map(|t| t.1.as_ref())
}

fn get_idx_by_id(
    step_vec: &TailDropVec<(String, Box<dyn Action>)>,
    step_id: &str,
) -> Option<usize> {
    for (idx, (sid, _)) in step_vec.iter().enumerate() {
        if sid == step_id {
            return Some(idx);
        }
    }
    return None;
}

pub fn next_step_id<'v, 's>(
    step_vec: &'v TailDropVec<(String, Box<dyn Action>)>,
    step_id: &str,
) -> Option<&'s str>
where
    'v: 's,
{
    let step_idx: usize = get_idx_by_id(step_vec, step_id)?;
    step_vec.get(step_idx + 1).map(|t| t.0.as_str())
}
