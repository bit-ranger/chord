use crate::model::PointContext;
use std::thread;
use async_std::sync::Arc;
use std::collections::HashMap;
use serde_json::Value;

pub async fn run_point(context: Arc<PointContext>) -> Result<(),()>{
    let url = context.get_config()["config"]["url"].as_str().unwrap();
    let url = context.render(url, Option::None);

    let resp = surf::get(&url)
        .header("Content-Type", "application/json")
        .recv_string()
        .await
        .unwrap();

    let json_value: serde_json::Result<Value> = serde_json::from_str(&resp);
    match json_value {
        Ok(value) => println!("{:?}", value),
        Err(e) => println!("{}", "not a json")
    }

    return Ok(());
}