use async_std::sync::Arc;
use futures::executor::block_on;
use log::{trace, warn};
use reqwest::Method;

use chord_core::value::{Map, Value};

use crate::docker::engine::Engine;
use crate::docker::Error;
use crate::docker::Error::*;

#[derive(Default)]
pub struct Arg {
    image: String,
    host_config: Option<Map>,
    env: Option<Vec<String>>,
    cmd: Option<Vec<String>>,
}

impl Arg {
    pub fn image(mut self, image: String) -> Arg {
        self.image = image;
        self
    }

    pub fn env(mut self, env: Vec<String>) -> Arg {
        self.env = Some(env);
        self
    }

    pub fn cmd(mut self, cmd: Vec<String>) -> Arg {
        self.cmd = Some(cmd);
        self
    }

    pub fn host_config(mut self, host_config: Map) -> Arg {
        self.host_config = Some(host_config);
        self
    }

    fn into_value(self) -> Value {
        let mut v = Map::new();
        v.insert("Image".to_string(), Value::String(self.image));
        if let Some(a) = self.host_config {
            v.insert("HostConfig".to_string(), Value::Object(a));
        }
        if let Some(a) = self.env {
            v.insert(
                "Env".to_string(),
                Value::Array(a.into_iter().map(|b| Value::String(b)).collect()),
            );
        }
        if let Some(a) = self.cmd {
            v.insert(
                "Cmd".to_string(),
                Value::Array(a.into_iter().map(|b| Value::String(b)).collect()),
            );
        }
        Value::Object(v)
    }
}

pub struct Container {
    engine: Arc<Engine>,
    name: String,
}

impl Container {
    pub async fn new(docker: Arc<Engine>, name: &str, arg: Arg) -> Result<Container, Error> {
        let arg = arg.into_value();
        trace!("container create {}, {}", name, arg);
        docker
            .call(
                format!("containers/create?name={}", name).as_str(),
                Method::POST,
                Some(arg),
                1,
            )
            .await
            .map(|_| Container {
                name: name.into(),
                engine: docker,
            })
    }

    pub async fn start(&mut self) -> Result<Vec<String>, Error> {
        trace!("container start {}", self.name);
        self.engine
            .call(
                format!("containers/{}/start", self.name).as_str(),
                Method::POST,
                None,
                1,
            )
            .await
    }

    pub async fn wait(&self) -> Result<Vec<String>, Error> {
        trace!("container wait {}", self.name);
        let res = self
            .engine
            .call(
                format!("containers/{}/wait", self.name).as_str(),
                Method::POST,
                None,
                1,
            )
            .await?;
        if res.len() == 1 && res[0].contains("\"StatusCode\":0") {
            return Ok(res);
        } else {
            return Err(Container(res[0].clone()));
        }
    }

    pub async fn tail(&self, stderr: bool, tail: usize) -> Result<Vec<String>, Error> {
        trace!("container log {}", self.name);
        if stderr {
            self.engine
                .call(
                    format!("containers/{}/logs?stderr=true&tail={}", self.name, tail).as_str(),
                    Method::GET,
                    None,
                    tail,
                )
                .await
        } else {
            self.engine
                .call(
                    format!("containers/{}/logs?stdout=true&tail={}", self.name, tail).as_str(),
                    Method::GET,
                    None,
                    tail,
                )
                .await
        }
    }
}

impl Drop for Container {
    fn drop(&mut self) {
        let uri = format!("containers/{}?force=true", self.name);
        let f = self.engine.call(uri.as_str(), Method::DELETE, None, 1);
        let _ = block_on(f)
            .map_err(|_| {
                warn!("container remove fail {}", self.name);
            })
            .map(|_| {
                trace!("container remove {}", self.name);
            });
    }
}
