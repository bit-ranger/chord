use crate::model::PointContext;
use std::thread;
use async_std::sync::Arc;
use std::collections::HashMap;
mod restapi;

async fn run_point_type(point_type: &str, context: Arc<PointContext>) ->  Result<(),()>{
    return if point_type.trim().eq("restapi") {
        restapi::run_point(context).await
    } else {
        Result::Err(())
    }


}

pub async fn run_point(context: Arc<PointContext>) -> Result<(),()>{
    let point_type = context.get_config()["type"].as_str().unwrap();
    run_point_type(point_type, context.clone()).await;

    let assert_condition = context.get_config()["assert"].as_str().unwrap();
    let assert_result = context.assert(assert_condition, Option::None);
    println!("run_point {} {} on thread {:?}", point_type, assert_result, thread::current().id());
    return Ok(());
}