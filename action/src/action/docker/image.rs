use surf::http::Method;

use chord::action::{Action, ActionValue, RunArg};
use chord::step::async_trait;
use chord::value::{from_str, json};
use chord::Error;

use crate::action::docker::container::Container;
use crate::action::docker::http::call;
use crate::action::docker::Docker;

pub struct Image {
    address: String,
    name: String,
}

#[async_trait]
impl Action for Image {
    async fn run(&self, arg: &dyn RunArg) -> ActionValue {
        let cmd = arg.render_value(&arg.config()["cmd"]).unwrap_or(json!([]));

        let mut container = Container::new(&self, arg.id().to_string().as_str(), cmd).await?;
        container.start().await?;

        let tail = arg
            .render_value(&arg.config()["tail"])?
            .as_u64()
            .unwrap_or(1) as usize;
        let tail_log = container.tail(tail).await?;

        Ok(from_str(tail_log.join("").as_str())?)
    }
}

impl Image {
    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    pub fn address(&self) -> &str {
        self.address.as_str()
    }

    pub async fn new(docker: &Docker, name: &str) -> Result<Image, Error> {
        let name = if name.contains(":") {
            name.into()
        } else {
            format!("{}:latest", name)
        };

        call(
            docker.address.as_str(),
            format!("images/create?fromImage={}", name).as_str(),
            Method::Post,
            None,
            1,
        )
        .await
        .map(|_| Image {
            address: docker.address.clone(),
            name,
        })
    }
}
