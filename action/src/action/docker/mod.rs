use chord::action::prelude::*;
use chord::err;
use image::Image;

mod container;
mod http;
mod image;

pub type Factory = Docker;

pub struct Docker {
    address: String,
}

impl Docker {
    pub async fn new(conf: Option<Value>) -> Result<Docker, Error> {
        let address: String = conf.map_or("".into(), |v| {
            v["address"].as_str().unwrap_or("127.0.0.1:2375").into()
        });
        Ok(Docker { address })
    }
}

#[async_trait]
impl ActionFactory for Docker {
    async fn create(&self, arg: &dyn CreateArg) -> Result<Box<dyn Action>, Error> {
        let image = arg.config()["image"]
            .as_str()
            .ok_or(err!("010", "missing image"))?;

        let image = Image::new(&self, image).await?;

        Ok(Box::new(image))
    }
}
