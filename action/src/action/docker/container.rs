use log::{trace, warn};
use surf::http::Method;

use chord::value::{json, Value};
use chord::Error;

use crate::action::docker::http::call;
use crate::action::docker::image::Image;
use futures::executor::block_on;

pub struct Container {
    address: String,
    name: String,
}

impl Container {
    pub async fn new(image: &Image, name: &str, cmd: Value) -> Result<Container, Error> {
        call(
            image.address(),
            format!("containers/create?name={}", name).as_str(),
            Method::Post,
            Some(json!({ "Image": image.name(),
                                "Cmd": cmd
            })),
            1,
        )
        .await
        .map(|_| {
            trace!("create {}", name);
            Container {
                name: name.into(),
                address: image.address().into(),
            }
        })
    }

    pub async fn start(&mut self) -> Result<(), Error> {
        call(
            self.address.as_str(),
            format!("containers/{}/start", self.name).as_str(),
            Method::Post,
            None,
            1,
        )
        .await
        .map(|_| {
            trace!("start {}", self.name);
        })
    }

    pub async fn tail(&self, tail: usize) -> Result<Vec<String>, Error> {
        call(
            self.address.as_str(),
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
            trace!("log {}", self.name);
            tl
        })
    }
}

impl Drop for Container {
    fn drop(&mut self) {
        let uri = format!("containers/{}?force=true", self.name);
        let f = call(self.address.as_str(), uri.as_str(), Method::Delete, None, 1);
        let _ = block_on(f)
            .map_err(|_| {
                warn!("remove fail {}", self.name);
            })
            .map(|_| {
                trace!("remove {}", self.name);
            });
    }
}
