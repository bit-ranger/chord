use serde_json::Value;

use crate::model::context::{PointContext, PointResult};

pub async fn run(context: &dyn PointContext) -> PointResult {
    let raw = context.get_config_rendered(vec!["raw"]).unwrap();
    let digest = md5::compute(raw);
    let digest = format!("{:x}", digest);
    return Ok(Value::String(digest));
}