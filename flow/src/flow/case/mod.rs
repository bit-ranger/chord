use chrono::Utc;
use log::{info, trace, warn};

use chord_core::case::CaseState;
use chord_core::collection::TailDropVec;
use chord_core::step::StepAssess;
use res::CaseAssessStruct;

use crate::flow::case::arg::CaseArgStruct;
use crate::flow::step::StepRunner;
use crate::model::app::FlowApp;

pub mod arg;
pub mod res;

pub async fn run(flow_ctx: &dyn FlowApp, arg: CaseArgStruct) -> CaseAssessStruct {
    trace!("case start {}", arg.id());
    let start = Utc::now();
    let mut step_assess_vec = Vec::<Box<dyn StepAssess>>::new();
    let step_vec = arg.step_vec().clone();

    for (step_id, step_runner) in step_vec.iter() {
        let step_runner: &StepRunner = step_runner;

        let mut step_arg = arg.step_arg_create(step_id, flow_ctx);

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
