use surf::http::Method;

use chord::action::prelude::*;
use chord::value::{from_str, json};
use chord::Error;

use crate::action::docker::container::Container;
use crate::action::docker::docker::Docker;
use async_std::sync::Arc;

pub struct Image {
    docker: Arc<Docker>,
    name: String,
}

#[async_trait]
impl Action for Image {
    async fn run(&self, arg: &dyn RunArg) -> ActionValue {
        let cmd = arg.render_value(&arg.config()["cmd"]).unwrap_or(json!([]));

        let mut container = Container::new(self.docker.clone(), &self, arg.id(), cmd).await?;
        container.start().await?;
        container.wait().await?;

        let tail = arg
            .render_value(&arg.config()["tail"])?
            .as_u64()
            .unwrap_or(1) as usize;
        let tail_log = container.tail(tail).await?;

        if tail_log.len() > 0 {
            Ok(from_str(tail_log.join("").as_str())?)
        } else {
            Ok(Value::Null)
        }
    }
}

impl Image {
    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    pub async fn new(docker: Arc<Docker>, name: &str) -> Result<Image, Error> {
        let name = if name.contains(":") {
            name.into()
        } else {
            format!("{}:latest", name)
        };

        docker
            .call(
                format!("images/create?fromImage={}", name).as_str(),
                Method::Post,
                None,
                1,
            )
            .await
            .map(|_| Image { docker, name })
    }
}
