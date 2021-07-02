use redis::{Client, Value as RedisValue};

use chord::action::prelude::*;
use chord::value::{from_str, Number};

pub struct RedisFactory {}

impl RedisFactory {
    pub async fn new(_: Option<Value>) -> Result<RedisFactory, Error> {
        Ok(RedisFactory {})
    }
}

#[async_trait]
impl Factory for RedisFactory {
    async fn create(&self, arg: &dyn CreateArg) -> Result<Box<dyn Action>, Error> {
        let url = arg.args()["url"]
            .as_str()
            .map(|s| arg.render_str(s))
            .ok_or(err!("010", "missing url"))??;

        if !arg.is_shared(url.as_str()) {
            return Ok(Box::new(Redis { client: None }));
        }

        let client = redis::Client::open(url)?;

        Ok(Box::new(Redis {
            client: Some(client),
        }))
    }
}

struct Redis {
    client: Option<Client>,
}

#[async_trait]
impl Action for Redis {
    async fn run(&self, arg: &dyn RunArg) -> Result<Box<dyn Scope>, Error> {
        return match self.client.as_ref() {
            Some(r) => run0(arg, r).await,
            None => {
                let url = arg.args()["url"]
                    .as_str()
                    .map(|s| arg.render_str(s))
                    .ok_or(err!("010", "missing url"))??;

                let client = redis::Client::open(url)?;
                run0(arg, &client).await
            }
        };
    }
}

async fn run0(arg: &dyn RunArg, client: &Client) -> Result<Box<dyn Scope>, Error> {
    let cmd = arg.args()["cmd"]
        .as_str()
        .map(|s| arg.render_str(s))
        .ok_or(err!("010", "missing cmd"))??;

    let mut con = client.get_async_connection().await?;

    let mut command = redis::cmd(cmd.as_str());
    let args_opt = arg.render_value(&arg.args()["args"])?;

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
