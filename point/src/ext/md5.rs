use common::point::{PointArg};
use common::value::Json;
use crate::model::PointValue;

pub async fn run(context: &dyn PointArg) -> PointValue {
    let raw = context.get_config_rendered(vec!["raw"]).unwrap();
    let digest = md5::compute(raw);
    let digest = format!("{:x}", digest);
    return Ok(Json::String(digest));
}