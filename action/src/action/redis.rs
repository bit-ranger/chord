use redis::{Client, Value as RedisValue};

use chord::action::prelude::*;

use crate::err;

pub struct RedisFactory {}

impl RedisFactory {
    pub async fn new(_: Option<Value>) -> Result<RedisFactory, Box<dyn Error>> {
        Ok(RedisFactory {})
    }
}

#[async_trait]
impl Factory for RedisFactory {
    async fn create(&self, arg: &dyn CreateArg) -> Result<Box<dyn Action>, Box<dyn Error>> {
        let args_raw = arg.args_raw();
        if let Some(url) = args_raw["url"].as_str() {
            if arg.is_static(url) {
                let url = arg.render_str(url)?;
                let url = url.as_str().ok_or(err!("100", "invalid url"))?;
                let client = redis::Client::open(url)?;
                return Ok(Box::new(Redis {
                    client: Some(client),
                }));
            }
        }
        return Ok(Box::new(Redis { client: None }));
    }
}

struct Redis {
    client: Option<Client>,
}

#[async_trait]
impl Action for Redis {
    async fn run(&self, arg: &dyn RunArg) -> Result<Box<dyn Scope>, Box<dyn Error>> {
        return match self.client.as_ref() {
            Some(r) => run0(arg, r).await,
            None => {
                let args = arg.args()?;
                let url = args["url"].as_str().ok_or(err!("101", "missing url"))?;

                let client = redis::Client::open(url)?;
                run0(arg, &client).await
            }
        };
    }
}

async fn run0(arg: &dyn RunArg, client: &Client) -> Result<Box<dyn Scope>, Box<dyn Error>> {
    let args = arg.args()?;
    let cmd = args["cmd"].as_str().ok_or(err!("102", "missing cmd"))?;

    let mut con = client.get_async_connection().await?;

    let mut command = redis::cmd(cmd);
    let args_opt = &args["args"];

    if let Some(arg_vec) = args_opt.as_array() {
        for a in arg_vec {
            command.arg(a.to_string());
        }
    }

    let redis_value: RedisValue = command.query_async(&mut con).await?;
    let result = match &redis_value {
        RedisValue::Nil => Value::Null,
        RedisValue::Int(i) => Value::Number(Number::from(i.clone())),
        RedisValue::Data(data) => {
            let data = String::from_utf8_lossy(data);
            let dv = from_str(data.as_ref());
            match dv {
                Ok(v) => v,
                Err(_) => Value::String(data.to_string()),
            }
        }
        RedisValue::Status(status) => Value::String(status.clone()),
        RedisValue::Okay => Value::String("OK".to_string()),
        _ => Value::Array(vec![]),
    };
    return Ok(Box::new(result));
}
