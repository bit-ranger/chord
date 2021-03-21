use common::point::PointArg;
use common::value::Json;
use crate::model::PointValue;
use async_std::task::sleep;
use std::time::Duration;

pub async fn run(point_arg: &dyn PointArg) -> PointValue {
    let seconds = point_arg.config()["seconds"].as_i64().unwrap_or(0) as u64;
    sleep(Duration::from_secs(seconds)).await;
    return Ok(Json::Null);
}