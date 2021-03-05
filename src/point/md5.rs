use crate::model::point::PointContext;
use crate::model::point::PointResult;
use serde_json::Value;


pub async fn run_point(context: & PointContext<'_, '_>) -> PointResult {
    let raw = context.get_config_str(vec!["raw"]).await.unwrap();
    let digest = md5::compute(raw.as_str());
    let digest = format!("{:x}", digest);
    return Ok(Value::String(digest));
}