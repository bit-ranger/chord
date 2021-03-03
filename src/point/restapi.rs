use crate::model::PointContext;
use crate::model::PointResult;
use std::thread;
use async_std::sync::Arc;
use std::collections::HashMap;
use serde_json::Value;
use serde::Serialize;
use std::borrow::Borrow;

pub async fn run_point(context: Arc<PointContext>) -> PointResult{
    let url = context.get_config_str(vec!["url"]).unwrap();

    let json :surf::Result<Value> = surf::get(&url)
        .header("Content-Type", "application/json")
        .recv_json()
        .await;

    match json {
        Ok(value) => println!("{}", value),
        Err(e) => println!("{}, {}, {}", url, "not a json", e)
    }

    return Ok(Value::Null);
}