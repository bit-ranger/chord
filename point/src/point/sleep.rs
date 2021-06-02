use async_std::task::sleep;
use chord_common::error::Error;
use chord_common::point::{async_trait, RunArg, PointRunner, PointValue, CreateArg};
use chord_common::value::Json;
use std::time::Duration;

struct Sleep {}

#[async_trait]
impl PointRunner for Sleep {
    async fn run(&self, arg: &dyn RunArg) -> PointValue {
        run(arg).await
    }
}

pub async fn create(_: Option<&Json>, _: &dyn CreateArg) -> Result<Box<dyn PointRunner>, Error> {
    Ok(Box::new(Sleep {}))
}

async fn run(arg: &dyn RunArg) -> PointValue {
    let seconds = arg.config()["seconds"].as_i64().unwrap_or(0) as u64;
    sleep(Duration::from_secs(seconds)).await;
    return Ok(Json::Null);
}
