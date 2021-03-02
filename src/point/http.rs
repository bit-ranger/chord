use crate::model::PointContext;
use std::thread;
use async_std::sync::Arc;
use std::collections::HashMap;

pub async fn run_point(context: Arc<PointContext>) -> Result<(),()>{
    let url = context.get_config()["config"]["url"].as_str().unwrap();
    return Ok(());
}