use serde_json::Value;
use crate::model::{PointResult, PointContext};


pub async fn run_point(context: &dyn PointContext) -> PointResult {
    let raw = context.get_config_str(vec!["raw"]).unwrap();
    let digest = md5::compute(raw);
    let digest = format!("{:x}", digest);
    return Ok(Value::String(digest));
}