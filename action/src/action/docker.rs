use std::sync::Arc;

use chord_core::action::prelude::*;
use chord_util::docker::container::Arg;
use chord_util::docker::engine::Engine;
use chord_util::docker::image::Image;

use crate::err;

pub struct Docker {
    engine: Arc<Engine>,
}

impl Docker {
    pub async fn new(conf: Option<Value>) -> Result<Docker, Error> {
        let address: String = conf.map_or("".into(), |v| {
            v["address"].as_str().unwrap_or("127.0.0.1:2375").into()
        });
        Ok(Docker {
            engine: Arc::new(Engine::new(address).await?),
        })
    }
}

#[async_trait]
impl Factory for Docker {
    async fn create(&self, arg: &dyn CreateArg) -> Result<Box<dyn Action>, Error> {
        let args_raw = arg.args_raw();
        let image = args_raw["image"]
            .as_str()
            .ok_or(err!("010", "missing image"))?;

        let image = Image::new(self.engine.clone(), image).await?;

        Ok(Box::new(ImageWrapper(image)))
    }
}

struct ImageWrapper(Image);

#[async_trait]
impl Action for ImageWrapper {
    async fn run(&self, arg: &dyn RunArg) -> Result<Box<dyn Scope>, Error> {
        let args = arg.args()?;
        let cmd = &args["cmd"];

        let mut ca = Arg::default();
        if cmd.is_array() {
            let cmd_vec = cmd
                .as_array()
                .unwrap()
                .iter()
                .map(|c| {
                    if c.is_string() {
                        c.as_str().unwrap().to_string()
                    } else {
                        c.to_string()
                    }
                })
                .collect();
            ca = ca.cmd(cmd_vec)
        }

        let mut container = self
            .0
            .container_create(arg.id().to_string().as_str(), ca)
            .await?;
        container.start().await?;
        let wait_res = container.wait().await;
        let tail = 1;
        let tail_lines = match wait_res {
            Ok(_) => container.tail(false, tail).await?,
            Err(_) => container.tail(true, tail).await?,
        };
        let tail_lines: Vec<String> = tail_lines
            .into_iter()
            .map(|row| row.trim().to_string())
            .filter(|row| !row.is_empty())
            .collect();

        if tail_lines.len() > 0 {
            let value_to_json = args["value_to_json"].as_bool().unwrap_or(false);
            if value_to_json {
                let value: Value = from_str(tail_lines.join("").as_str())?;
                Ok(Box::new(value))
            } else {
                let value: Value = Value::String(tail_lines.join(""));
                Ok(Box::new(value))
            }
        } else {
            Ok(Box::new(Value::Null))
        }
    }
}
