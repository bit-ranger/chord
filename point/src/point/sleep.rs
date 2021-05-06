use chord_common::point::{PointArg, PointValue, PointRunner, Pin, Future};
use chord_common::value::Json;
use async_std::task::sleep;
use std::time::Duration;
use chord_common::error::Error;

struct Sleep {}

impl PointRunner for Sleep {

    fn run<'a>(&self, arg: &'a dyn PointArg) -> Pin<Box<dyn Future<Output=PointValue> + Send + 'a>> {
        Box::pin(run(arg))
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

