use common::point::PointArg;
use common::value::{Json, Map};
use crate::model::{PointValue, PointError};
use redis::{RedisError, Value as RedisValue, FromRedisValue};
use crate::perr;
use std::collections::HashMap;

pub async fn run(pt_arg: &dyn PointArg) -> PointValue {
    let url = pt_arg.config_rendered(vec!["url"]).ok_or(perr!("010", "missing url"))?;
    let cmd = pt_arg.config_rendered(vec!["cmd"]).ok_or(perr!("012", "missing cmd"))?;

    let client = redis::Client::open(url)?;
    let mut con = client.get_async_connection().await?;

    let mut command = redis::cmd(cmd.as_str());
    let args_opt = pt_arg.config()["args"].as_array();
    if args_opt.is_some(){
        for arg in args_opt.unwrap() {
            command.arg(arg.to_string().as_str());
        }
    }

    let redis_value:RedisValue = command.query_async(&mut con).await?;
    let result = match &redis_value {
        RedisValue::Nil => {
            Json::Null
        },
        _ => {
            let hash_map = HashMap::<String,String>::from_redis_value(&redis_value)?;
            let mut map = Map::new();
            for (k, v) in hash_map.into_iter() {
                map.insert(k, Json::String(v));
            }
            Json::Object(map)
        }
    };

    println!("{}", result);
    return Ok(result);
}

impl From<RedisError> for PointError {
    fn from(err: RedisError) -> PointError {
        PointError::new("redis", format!("{:?}", err).as_str())
    }
}