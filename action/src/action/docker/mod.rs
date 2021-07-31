use crate::action::docker::engine::Engine;
use async_std::sync::Arc;
use chord::action::prelude::*;
use image::Image;

mod container;
mod engine;
mod image;

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
        let image = arg.args_raw()["image"]
            .as_str()
            .ok_or(err!("010", "missing image"))?;

        let image = Image::new(self.engine.clone(), image).await?;

        Ok(Box::new(image))
    }
}
