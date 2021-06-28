use async_std::sync::Arc;
use futures::executor::block_on;
use log::{trace, warn};
use surf::http::Method;

use chord::value::{json, Value};
use chord::Error;

use crate::action::docker::docker::Docker;
use crate::action::docker::image::Image;

pub struct Container {
    docker: Arc<Docker>,
    name: String,
}

impl Container {
    pub async fn new(
        docker: Arc<Docker>,
        image: &Image,
        name: &str,
        cmd: Value,
    ) -> Result<Container, Error> {
        docker
            .call(
                format!("containers/create?name={}", name).as_str(),
                Method::Post,
                Some(json!({ "Image": image.name(),
                                    "Cmd": cmd
                })),
                1,
            )
            .await
            .map(|_| {
                trace!("container create {}", name);
                Container {
                    name: name.into(),
                    docker,
                }
            })
    }

    pub async fn start(&mut self) -> Result<(), Error> {
        self.docker
            .call(
                format!("containers/{}/start", self.name).as_str(),
                Method::Post,
                None,
                1,
            )
            .await
            .map(|_| {
                trace!("container start {}", self.name);
            })
    }

    pub async fn tail(&self, tail: usize) -> Result<Vec<String>, Error> {
        self.docker
            .call(
                format!(
                    "containers/{}/logs?stdout=true&stderr=true&tail={}",
                    self.name, tail
                )
                .as_str(),
                Method::Get,
                None,
                tail,
            )
            .await
            .map(|tl| {
                trace!("container log {}", self.name);
                tl
            })
    }
}

impl Drop for Container {
    fn drop(&mut self) {
        let uri = format!("containers/{}?force=true", self.name);
        let f = self.docker.call(uri.as_str(), Method::Delete, None, 1);
        let _ = block_on(f)
            .map_err(|_| {
                warn!("container remove fail {}", self.name);
            })
            .map(|_| {
                trace!("container remove {}", self.name);
            });
    }
}
