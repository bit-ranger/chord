use chord_common::point::{PointArg, PointValue, PointRunner, async_trait};
use chord_common::value::Json;
use async_std::task::sleep;
use std::time::Duration;
use chord_common::error::Error;

struct Sleep {}

#[async_trait]
impl PointRunner for Sleep {

    async fn run(&self, arg: &dyn PointArg) -> PointValue {
        run(arg).await
    }
}

pub async fn create(_: &Json) -> Result<Box<dyn PointRunner>, Error>{
    Ok(Box::new(Sleep {}))
}


async fn run(arg: &dyn PointArg) -> PointValue {
    let seconds = arg.config()["seconds"].as_i64().unwrap_or(0) as u64;
    sleep(Duration::from_secs(seconds)).await;
    return Ok(Json::Null);
}

