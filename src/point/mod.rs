use crate::model::PointContext;
use std::thread;
use async_std::sync::{Arc, RwLock};
use std::collections::HashMap;
use crate::model::PointResult;
use serde::Serialize;
use std::borrow::Borrow;
use serde_json::Value;
use std::ops::{Deref, DerefMut};

mod restapi;
mod md5;

async fn run_point_type(point_type: &str, context: &PointContext) ->  PointResult
{
    return if point_type.trim().eq("restapi") {
        restapi::run_point(context).await
    }else if point_type.trim().eq("md5") {
        md5::run_point(context).await
    } else {
        PointResult::Err(())
    }
}

pub async fn run_point(context: Arc<RwLock<PointContext>>) -> PointResult
{

    let point_type = context.read().await.deref().get_meta_str(vec!["type"]).unwrap();
    let result = run_point_type(
        point_type.as_str(),
        context.read().await.deref())
        .await;
    if result.is_err() {
        return PointResult::Err(());
    }

    let value = result.unwrap();
    context.write().await.deref_mut().register_context(String::from("result"), &value);
    let assert_condition = context.read().await.deref().get_meta_str(vec!["assert"]);
    match assert_condition{
        Some(con) =>  {
            let assert_result = context.read().await.deref().assert(con.as_str(), &Value::Null);
            if assert_result {PointResult::Ok(value)} else {PointResult::Err(())}
        },
        None => return Ok(Value::Null)
    }
}

