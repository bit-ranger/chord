use async_std::sync::Arc;
use futures::executor::block_on;
use log::{trace, warn};
use surf::http::Method;

use chord::action::prelude::*;
use chord::value::from_str;
use chord::Error;

use crate::action::docker::container::Container;
use crate::action::docker::engine::Engine;

pub struct Image {
    engine: Arc<Engine>,
    name: String,
}

#[async_trait]
impl Action for Image {
    async fn run(&self, arg: &dyn RunArg) -> Result<Box<dyn Scope>, Error> {
        let args = arg.args(None)?;
        let cmd = &args["cmd"];

        let mut container = Container::new(
            self.engine.clone(),
            &self,
            arg.id().to_string().as_str(),
            cmd.clone(),
        )
        .await?;
        container.start().await?;
        container.wait().await?;

        let tail = args["tail"].as_u64().unwrap_or(1) as usize;
        let tail_log = container.tail(tail).await?;
        let tail_log: Vec<String> = tail_log
            .into_iter()
            .map(|row| row.trim().to_string())
            .filter(|row| !row.is_empty())
            .collect();

        if tail_log.len() > 0 {
            let value: Value = from_str(tail_log.join("").as_str())?;
            Ok(Box::new(value))
        } else {
            Ok(Box::new(Value::Null))
        }
    }
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
