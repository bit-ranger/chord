use crate::model::PointContext;
use crate::model::PointResult;
use std::thread;
use async_std::sync::Arc;
use std::collections::HashMap;
use serde_json::Value;
use serde::Serialize;


pub async fn run_point(context: Arc<PointContext>) -> PointResult {
    let raw = context.get_config_str(vec!["raw"]).unwrap();
    let digest = md5::compute(raw.as_str());
    let digest = format!("{:x}", digest);
    return Ok(Value::String(digest));
}