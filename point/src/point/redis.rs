use chord_common::err;
use chord_common::error::Error;
use chord_common::point::{async_trait, PointArg, PointRunner, PointValue};
use chord_common::value::{from_str, Json, Number};
use redis::Value as RedisValue;

struct Redis {}

#[async_trait]
impl PointRunner for Redis {
    async fn run(&self, arg: &dyn PointArg) -> PointValue {
        run(arg).await
    }
}

pub async fn create(_: &dyn PointArg) -> Result<Box<dyn PointRunner>, Error> {
    Ok(Box::new(Redis {}))
}

async fn run(arg: &dyn PointArg) -> PointValue {
    let url = arg.config()["url"]
        .as_str()
        .map(|s| arg.render(s))
        .ok_or(err!("010", "missing url"))??;
    let cmd = arg.config()["cmd"]
        .as_str()
        .map(|s| arg.render(s))
        .ok_or(err!("010", "missing cmd"))??;

    let client = redis::Client::open(url)?;
    let mut con = client.get_async_connection().await?;

    let mut command = redis::cmd(cmd.as_str());
    let args_opt = arg.config()["args"].as_array();
    if args_opt.is_some() {
        for arg in args_opt.unwrap() {
            command.arg(arg.to_string().as_str());
        }
    }

    let redis_value: RedisValue = command.query_async(&mut con).await?;
    let result = match &redis_value {
        RedisValue::Nil => Json::Null,
        RedisValue::Int(i) => Json::Number(Number::from(i.clone())),
        RedisValue::Data(data) => {
            let data = String::from_utf8_lossy(data);
            let dv = from_str(data.as_ref());
            match dv {
                Ok(v) => v,
                Err(_) => Json::String(data.to_string()),
            }
        }
        RedisValue::Status(status) => Json::String(status.clone()),
        RedisValue::Okay => Json::String("OK".to_string()),
        _ => Json::Array(vec![]),
    };
    return Ok(result);
}
