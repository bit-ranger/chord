use async_std::sync::Arc;
use chord::action::prelude::*;
use chord_util::docker::container::Arg;
use chord_util::docker::engine::Engine;
use chord_util::docker::image::Image;

pub struct Docker {
    engine: Arc<Engine>,
}

impl Docker {
    pub async fn new(conf: Option<Value>) -> Result<Docker, Error> {
        let address: String = conf.map_or("".into(), |v| {
            v["address"].as_str().unwrap_or("127.0.0.1:2375").into()
        });
        Ok(Docker {
            engine: Arc::new(Engine::new(address).await?),
        })
    }
}

#[async_trait]
impl Factory for Docker {
    async fn create(&self, arg: &dyn CreateArg) -> Result<Box<dyn Action>, Error> {
        let image = arg.args_raw()["image"]
            .as_str()
            .ok_or(err!("010", "missing image"))?;

        let image = Image::new(self.engine.clone(), image).await?;

        Ok(Box::new(ImageWrapper(image)))
    }
}

struct ImageWrapper(Image);

#[async_trait]
impl Action for ImageWrapper {
    async fn run(&self, arg: &dyn RunArg) -> Result<Box<dyn Scope>, Error> {
        let args = arg.args(None)?;
        let cmd = &args["cmd"];

        let mut ca = Arg::default();
        if cmd.is_array() {
            let cmd_vec = cmd
                .as_array()
                .unwrap()
                .iter()
                .map(|c| c.to_string())
                .collect();
            ca = ca.cmd(cmd_vec)
        }

        let mut container = self
            .0
            .container_create(arg.id().to_string().as_str(), ca)
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
