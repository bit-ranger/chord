use common::point::PointArg;
use common::value::Json;
use crate::model::PointValue;
use async_std::task::sleep;
use std::time::Duration;
use crate::ext::config_rendered_default;

pub async fn run(point_arg: &dyn PointArg) -> PointValue {
    let seconds = config_rendered_default(point_arg, vec!["seconds"], 5)?;
    sleep(Duration::from_secs(seconds));
    return Ok(Json::Null);
}