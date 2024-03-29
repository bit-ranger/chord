use std::sync::Arc;

use log::{trace, warn};
use reqwest::Method;

use chord_core::future::task::spawn;

use crate::docker::container::{Arg, Container};
use crate::docker::engine::Engine;
use crate::docker::Error;
use crate::docker::Error::*;

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
                Method::POST,
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
        let name = self.name.clone();
        let engine = self.engine.clone();
        let _ = spawn(async move {
            engine
                .call(uri.as_str(), Method::DELETE, None, 1)
                .await
                .map_err(|e| {
                    if let Status(404) = e {
                        trace!("image not found {}", name);
                    } else {
                        warn!("image remove fail {}, {}", name, e);
                    }
                })
                .map(|_| {
                    trace!("image remove {}", name);
                })
        });
    }
}
