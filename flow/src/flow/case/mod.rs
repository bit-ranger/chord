use chrono::Utc;
use log::{info, trace, warn};
use tracing::{error_span, Instrument};

use chord_core::case::CaseState;
use chord_core::collection::TailDropVec;
use chord_core::step::StepAsset;
use res::CaseAssetStruct;

use crate::flow::case::arg::CaseArgStruct;
use crate::flow::step::StepRunner;
use crate::model::app::App;

pub mod arg;
pub mod res;

pub async fn run(flow_ctx: &dyn App, mut arg: CaseArgStruct) -> CaseAssetStruct {
    trace!("case run");
    let start = Utc::now();
    let mut step_asset_vec = Vec::<Box<dyn StepAsset>>::new();
    let step_vec = arg.step_vec().clone();

    for (step_id, step_runner) in step_vec.iter() {
        let step_runner: &StepRunner = step_runner;

        let mut step_arg = arg.step_arg_create(step_id, flow_ctx);

        let step_asset = step_runner.run(&mut step_arg)
            .instrument(error_span!("step", id=step_id))
            .await;

        if !step_asset.state().is_ok() {
            step_asset_vec.push(Box::new(step_asset));
            warn!("case Fail");
            return CaseAssetStruct::new(
                arg.id().clone(),
                start,
                Utc::now(),
                arg.take_data(),
                CaseState::Fail(TailDropVec::from(step_asset_vec)),
            );
        } else {
            arg.step_asset_register(step_asset.id().step(), &step_asset)
                .await;
            step_asset_vec.push(Box::new(step_asset));
        }
    }

    info!("case Ok");
    return CaseAssetStruct::new(
        arg.id().clone(),
        start,
        Utc::now(),
        arg.take_data(),
        CaseState::Ok(TailDropVec::from(step_asset_vec)),
    );
}
