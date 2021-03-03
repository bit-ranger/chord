use crate::model::PointContext;
use std::thread;
use async_std::sync::Arc;
use std::collections::HashMap;

pub async fn run_point(context: Arc<PointContext>) -> Result<(),()>{
    let url = context.get_config()["config"]["url"].as_str().unwrap();
    let url = context.render(url, Option::None);

    let json :surf::Result<HashMap<String,String>> = surf::get(&url)
        .header("Content-Type", "application/json")
        .recv_json()
        .await;

    match json {
        Ok(value) => println!("{:?}", value),
        Err(e) => println!("{}", "not a json")
    }

    return Ok(());
}