use chrono::{DateTime, Utc};
use handlebars::TemplateRenderError;
use log::{error, info, trace, warn};

use chord_core::action::Action;
use chord_core::case::CaseState;
use chord_core::collection::TailDropVec;
use chord_core::step::{StepAssess, StepState};
use chord_core::value::Value;
use Error::*;
use res::CaseAssessStruct;

use crate::flow::case::arg::CaseArgStruct;
use crate::flow::step;
use crate::flow::step::arg::RunIdStruct;
use crate::flow::step::res::{ActionAssessStruct, StepAssessStruct};
use crate::flow::step::StepRunner;
use crate::model::app::FlowApp;

pub mod arg;
pub mod res;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("unrecognized step_id: `{0}`")]
    StepId(String),

    #[error("`{0}` render error:\n{1}")]
    Render(String, TemplateRenderError),
}

pub async fn run(flow_ctx: &dyn FlowApp, mut arg: CaseArgStruct) -> CaseAssessStruct {
    trace!("case start {}", arg.id());
    let start = Utc::now();
    let mut step_assess_vec = Vec::<Box<dyn StepAssess>>::new();
    let step_vec = arg.step_vec().clone();

    for (step_id, step_runner) in step_vec.iter() {
        let step_runner: &StepRunner = step_runner;

        let step_arg = arg.step_arg_create(step_id, flow_ctx);


        if let Err(e) = step_arg {
            return case_fail_by_step_err(step_id.as_str(), arg, e, step_assess_vec, start);
        }
        let mut step_arg = step_arg.unwrap();


        let step_assess = step_runner.run(&mut step_arg).await;

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
            step_assess_vec.push(Box::new(step_assess));
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
    let step_run_id = RunIdStruct::new(step_id.to_string(), arg.id());
    error!("step Err {}\n{}", step_run_id, e);

    let step_assess = StepAssessStruct::new(
        step_run_id,
        Utc::now(),
        Utc::now(),
        vec![ActionAssessStruct::new(
            Utc::now(),
            Utc::now(),
            Value::Null,
            StepState::Err(Box::new(e)),
        )],
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
