use chord_common::value::Json;
use chord_common::point::{PointArg, PointValue};

pub async fn run(pt_arg: &dyn PointArg) -> PointValue {
    let raw = pt_arg.config_rendered(vec!["raw"]).unwrap();
    let digest = md5::compute(raw);
    let digest = format!("{:x}", digest);
    return Ok(Json::String(digest));
}