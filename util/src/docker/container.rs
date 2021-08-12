use async_std::sync::Arc;
use futures::executor::block_on;
use log::{trace, warn};
use surf::http::Method;

use chord::value::{Map, Value};
use chord::Error;

use crate::docker::engine::Engine;
use serde::Serialize;

#[derive(Serialize, Default)]
pub struct Arg {
    image: String,
    volumes: Option<Map>,
    env: Option<Map>,
    cmd: Option<Vec<Value>>,
}

impl Arg {
    pub fn image(mut self, image: String) -> Arg {
        self.image = image;
        self
    }

    pub fn volumes(mut self, volumes: Map) -> Arg {
        self.volumes = Some(volumes);
        self
    }

    pub fn env(mut self, env: Map) -> Arg {
        self.env = Some(env);
        self
    }

    pub fn cmd(mut self, cmd: Vec<Value>) -> Arg {
        self.cmd = Some(cmd);
        self
    }

    fn into_value(self) -> Value {
        let mut v = Map::new();
        v.insert("Image".to_string(), Value::String(self.image));
        if let Some(a) = self.volumes {
            v.insert("Volumes".to_string(), Value::Object(a));
        }
        if let Some(a) = self.env {
            v.insert("Env".to_string(), Value::Object(a));
        }
        if let Some(a) = self.cmd {
            v.insert("Cmd".to_string(), Value::Array(a));
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
                Method::Post,
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
                Method::Post,
                None,
                1,
            )
            .await
    }

    pub async fn wait(&self) -> Result<Vec<String>, Error> {
        trace!("container wait {}", self.name);
        self.engine
            .call(
                format!("containers/{}/wait", self.name).as_str(),
                Method::Post,
                None,
                1,
            )
            .await
    }

    pub async fn tail(&self, tail: usize) -> Result<Vec<String>, Error> {
        trace!("container log {}", self.name);
        self.engine
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
    }
}

impl Drop for Container {
    fn drop(&mut self) {
        let uri = format!("containers/{}?force=true", self.name);
        let f = self.engine.call(uri.as_str(), Method::Delete, None, 1);
        let _ = block_on(f)
            .map_err(|_| {
                warn!("container remove fail {}", self.name);
            })
            .map(|_| {
                trace!("container remove {}", self.name);
            });
    }
}
