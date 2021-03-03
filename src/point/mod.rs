use crate::model::PointContext;
use std::thread;
use async_std::sync::Arc;
use std::collections::HashMap;
use crate::model::PointResult;
use serde::Serialize;
use std::borrow::Borrow;
use serde_json::Value;

mod restapi;
mod md5;

async fn run_point_type(point_type: &str, context: Arc<PointContext>) ->  PointResult
{
    return if point_type.trim().eq("restapi") {
        restapi::run_point(context).await
    }else if point_type.trim().eq("md5") {
        md5::run_point(context).await
    } else {
        PointResult::Err(())
    }
}

pub async fn run_point(context: Arc<PointContext>) -> PointResult
{
    let point_type = context.get_meta_str(vec!["type"]).unwrap();
    let result = run_point_type(point_type.as_str(), context.clone()).await;

    let assert_condition = context.get_meta_str(vec!["assert"]);
    match assert_condition{
        Some(con) =>  {
            let assert_result = context.assert(con.as_str(), &Value::Null);
            if assert_result {result} else {Err(())}
        },
        None => return Ok(Value::Null)
    }
}