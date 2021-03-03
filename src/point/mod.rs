use crate::model::PointContext;
use std::thread;
use async_std::sync::Arc;
use std::collections::HashMap;
use serde_yaml::Value;

mod restapi;

async fn run_point_type(point_type: &str, context: Arc<PointContext>) ->  Result<(),()>{
    return if point_type.trim().eq("restapi") {
        restapi::run_point(context).await
    } else {
        Result::Err(())
    }


}

pub async fn run_point(context: Arc<PointContext>) -> Result<(),()>{
    let point_type = context.get_config_str(vec!["type"]).unwrap();
    run_point_type(point_type.as_str(), context.clone()).await;

    let assert_condition = context.get_config_str(vec!["assert"]).unwrap();
    let assert_result = context.assert(assert_condition.as_str(), &Value::Null);
    // println!("run_point {} {} on thread {:?}", point_type, assert_result, thread::current().id());
    return Ok(());
}