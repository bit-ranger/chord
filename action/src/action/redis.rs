use redis::{Client, Value as RedisValue};


use chord_core::action::prelude::*;

use crate::err;

pub struct RedisCreator {}

impl RedisCreator {
    pub async fn new(_: Option<Value>) -> Result<RedisCreator, Error> {
        Ok(RedisCreator {})
    }
}

#[async_trait]
impl Creator for RedisCreator {
    async fn create(&self, chord: &dyn Chord, arg: &dyn Arg) -> Result<Box<dyn Action>, Error> {
        let args_init = arg.args_init();
        if let Some(args_init) = args_init {
            let url = &args_init["url"];
            if url.is_string() {
                let url = chord.render(arg.context(), url)?;
                let url = url.as_str().ok_or(err!("100", "invalid url"))?;
                let client = Client::open(url)?;
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
    async fn execute(
        &self,
        chord: &dyn Chord,
        arg: &mut dyn Arg,
    ) -> Result<Asset, Error> {
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

async fn run0(arg: &dyn Arg, client: &Client) -> Result<Asset, Error> {
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
    return Ok(Asset::Value(result));
}
