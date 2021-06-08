use chord_common::err;
use chord_common::error::Error;
use chord_common::step::{
    async_trait, CreateArg, RunArg, StepRunner, StepRunnerFactory, StepValue,
};
use chord_common::value::{from_str, Json, Number};
use redis::Value as RedisValue;

pub struct Factory {}

impl Factory {
    pub async fn new(_: Option<Json>) -> Result<Factory, Error> {
        Ok(Factory {})
    }
}

#[async_trait]
impl StepRunnerFactory for Factory {
    async fn create(&self, _: &dyn CreateArg) -> Result<Box<dyn StepRunner>, Error> {
        Ok(Box::new(Runner {}))
    }
}

struct Runner {}

#[async_trait]
impl StepRunner for Runner {
    async fn run(&self, arg: &dyn RunArg) -> StepValue {
        run(arg).await
    }
}

async fn run(arg: &dyn RunArg) -> StepValue {
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
