use async_std::sync::Arc;
use futures::executor::block_on;
use log::{trace, warn};
use surf::http::Method;

use crate::docker::container::{Arg, Container};
use crate::docker::engine::Engine;
use crate::docker::Error;

pub struct Image {
    engine: Arc<Engine>,
    name: String,
}

impl Image {
    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    pub async fn new(engine: Arc<Engine>, name: &str) -> Result<Image, Error> {
        let name = if name.contains(":") {
            name.into()
        } else {
            format!("{}:latest", name)
        };

        trace!("image create {}", name);
        engine
            .call(
                format!("images/create?fromImage={}", name).as_str(),
                Method::Post,
                None,
                1,
            )
            .await
            .map(|_| Image { engine, name })
    }

    pub async fn container_create(&self, name: &str, arg: Arg) -> Result<Container, Error> {
        let arg = arg.image(self.name.clone());
        return Container::new(self.engine.clone(), name, arg).await;
    }
}

impl Drop for Image {
    fn drop(&mut self) {
        let uri = format!("images/{}", self.name);
        let f = self.engine.call(uri.as_str(), Method::Delete, None, 1);
        let _ = block_on(f)
            .map_err(|e| {
                if e.code() == "docker" && e.message() == "404" {
                    trace!("image not found {}", self.name);
                } else {
                    warn!("image remove fail {}, {}", self.name, e);
                }
            })
            .map(|_| {
                trace!("image remove {}", self.name);
            });
    }
}
