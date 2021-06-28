use crate::action::docker::docker::Docker;
use async_std::sync::Arc;
use chord::action::prelude::*;
use chord::err;
use image::Image;

mod container;
mod docker;
mod image;

pub struct Factory {
    docker: Arc<Docker>,
}

impl Factory {
    pub async fn new(conf: Option<Value>) -> Result<Factory, Error> {
        let address: String = conf.map_or("".into(), |v| {
            v["address"].as_str().unwrap_or("127.0.0.1:2375").into()
        });
        Ok(Factory {
            docker: Arc::new(Docker::new(address).await?),
        })
    }
}

#[async_trait]
impl ActionFactory for Factory {
    async fn create(&self, arg: &dyn CreateArg) -> Result<Box<dyn Action>, Error> {
        let image = arg.config()["image"]
            .as_str()
            .ok_or(err!("010", "missing image"))?;

        let image = Image::new(self.docker.clone(), image).await?;

        Ok(Box::new(image))
    }
}
