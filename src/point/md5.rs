use serde_json::Value;

use crate::model::point::PointArg;
use crate::model::point::PointValue;

pub async fn run(context: &dyn PointArg) -> PointValue {
    let raw = context.get_config_rendered(vec!["raw"]).unwrap();
    let digest = md5::compute(raw);
    let digest = format!("{:x}", digest);
    return Ok(Value::String(digest));
}