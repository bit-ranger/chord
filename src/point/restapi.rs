use crate::model::PointContext;
use std::thread;
use async_std::sync::Arc;
use std::collections::HashMap;
use serde_json::Value;

pub async fn run_point(context: Arc<PointContext>) -> Result<(),()>{
    let url = context.get_config_str(vec!["config", "url"]).unwrap();

    let json :surf::Result<Value> = surf::get(&url)
        .header("Content-Type", "application/json")
        .recv_json()
        .await;

    match json {
        Ok(value) => println!("{}", value),
        Err(e) => println!("{}, {}, {}", url, "not a json", e)
    }

    return Ok(());
}