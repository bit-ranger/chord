use std::borrow::{Borrow, BorrowMut};
use std::sync::Arc;

use chord_core::action::prelude::*;
use chord_core::future::sync::RwLock;
use chord_util::docker::container::Arg as DkArg;
use chord_util::docker::engine::Engine;
use chord_util::docker::image::Image;

use crate::err;

pub struct Docker {
    actual: RwLock<Option<DockerActual>>,
    address: String,
}

impl Docker {
    pub async fn new(conf: Option<Value>) -> Result<Docker, Error> {
        let address: String = conf.map_or("".into(), |v| {
            v["address"].as_str().unwrap_or("127.0.0.1:2375").into()
        });
        Ok(Docker {
            actual: RwLock::new(None),
            address,
        })
    }
}

#[async_trait]
impl Creator for Docker {
    async fn create(&self, chord: &dyn Chord, arg: &dyn Arg) -> Result<Box<dyn Action>, Error> {
        let creator = self.actual.read().await;
        let creator_ref = creator.borrow();
        if creator_ref.is_some() {
            return creator_ref.as_ref().unwrap().create(chord, arg).await;
        } else {
            drop(creator);
            let mut guard = self.actual.write().await;
            let guard = guard.borrow_mut();
            let new_creator = DockerActual::new(self.address.clone()).await?;
            let action = new_creator.create(chord, arg).await?;
            guard.replace(new_creator);
            return Ok(action);
        }
    }
}

pub struct DockerActual {
    engine: Arc<Engine>,
}

impl DockerActual {
    async fn new(address: String) -> Result<DockerActual, Error> {
        Ok(DockerActual {
            engine: Arc::new(Engine::new(address).await?),
        })
    }
}

#[async_trait]
impl Creator for DockerActual {
    async fn create(&self, _chord: &dyn Chord, arg: &dyn Arg) -> Result<Box<dyn Action>, Error> {
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
    async fn execute(
        &self,
        _chord: &dyn Chord,
        arg: &mut dyn Arg,
    ) -> Result<Box<dyn Scope>, Error> {
        let args = arg.args()?;
        let cmd = &args["cmd"];

        let mut ca = DkArg::default();
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
