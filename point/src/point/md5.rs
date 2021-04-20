use chord_common::value::Json;
use chord_common::point::{PointArg, PointValue};

pub async fn run(context: &dyn PointArg) -> PointValue {
    let raw = context.config_rendered(vec!["raw"]).unwrap();
    let digest = md5::compute(raw);
    let digest = format!("{:x}", digest);
    return Ok(Json::String(digest));
}