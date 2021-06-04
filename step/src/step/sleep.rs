use async_std::task::sleep;
use chord_common::error::Error;
use chord_common::step::{async_trait, CreateArg, RunArg, StepRunner, StepValue};
use chord_common::value::Json;
use std::time::Duration;

struct Sleep {}

#[async_trait]
impl StepRunner for Sleep {
    async fn run(&self, arg: &dyn RunArg) -> StepValue {
        run(arg).await
    }
}

pub async fn create(_: Option<&Json>, _: &dyn CreateArg) -> Result<Box<dyn StepRunner>, Error> {
    Ok(Box::new(Sleep {}))
}

async fn run(arg: &dyn RunArg) -> StepValue {
    let seconds = arg.config()["duration"].as_u64().unwrap_or(0) as u64;
    sleep(Duration::from_secs(seconds)).await;
    return Ok(Json::Null);
}
