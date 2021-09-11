use chrono::{DateTime, Utc};
use log::{debug, info, trace};

use chord::case::CaseState;
use chord::collection::TailDropVec;
use chord::step::{StepAssess, StepState};
use res::CaseAssessStruct;

use crate::flow::case::arg::CaseArgStruct;
use crate::flow::step;
use crate::flow::step::arg::RunIdStruct;
use crate::flow::step::res::StepAssessStruct;
use crate::model::app::FlowApp;
use chord::Error;

pub mod arg;
pub mod res;

pub async fn run(flow_ctx: &dyn FlowApp, mut arg: CaseArgStruct) -> CaseAssessStruct {
    trace!("case start {}", arg.id());
    let start = Utc::now();
    let mut step_assess_vec = Vec::<Box<dyn StepAssess>>::new();
    for (step_id, action) in arg.step_vec().iter() {
        let step_arg = arg.step_arg_create(step_id, flow_ctx);
        if let Err(e) = step_arg {
            return case_fail_by_step_err(step_id, arg, e, step_assess_vec, start);
        }
        let step_arg = step_arg.unwrap();
        let step_assess = step::run(flow_ctx, &step_arg, action.as_ref()).await;

        if !step_assess.state.is_ok() {
            step_assess_vec.push(Box::new(step_assess));
            debug!("case Fail  {}", arg.id());
            return CaseAssessStruct::new(
                arg.id().clone(),
                start,
                Utc::now(),
                arg.take_data(),
                CaseState::Fail(TailDropVec::from(step_assess_vec)),
            );
        } else {
            arg.step_ok_register(step_assess.id().step(), step_assess.state())
                .await;
            step_assess_vec.push(Box::new(step_assess));
        }
    }

    debug!("case Ok   {}", arg.id());
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
    info!("step Err {}\n{}", step_id, e);

    let step_run_id = RunIdStruct::new(step_id.to_string(), arg.id());
    let step_assess = StepAssessStruct::new(step_run_id, Utc::now(), Utc::now(), StepState::Err(e));
    step_assess_vec.push(Box::new(step_assess));
    info!("case Fail {}", arg.id());
    return CaseAssessStruct::new(
        arg.id().clone(),
        start,
        Utc::now(),
        arg.take_data(),
        CaseState::Fail(TailDropVec::from(step_assess_vec)),
    );
}
