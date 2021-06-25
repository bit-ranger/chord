use async_std::prelude::*;
use chord::step::{async_trait, Action, ActionFactory, ActionValue, CreateArg, RunArg};
use chord::value::{from_str, json, Value};
use chord::Error;
use chord::{cause, err, rcause, rerr};
use log::trace;
use std::str::FromStr;
use surf::http::headers::{HeaderName, HeaderValue};
use surf::http::Method;
use surf::{RequestBuilder, Response, Url};

pub struct Factory {}

impl Factory {
    pub async fn new(_: Option<Value>) -> Result<Factory, Error> {
        Ok(Factory {})
    }
}

#[async_trait]
impl ActionFactory for Factory {
    async fn create(&self, arg: &dyn CreateArg) -> Result<Box<dyn Action>, Error> {
        create0(self, arg).await.map_err(|e| e.0)
    }
}

struct Runner {
    image: String,
}

#[async_trait]
impl Action for Runner {
    async fn run(&self, arg: &dyn RunArg) -> ActionValue {
        run0(self, arg).await.map_err(|e| e.0)
    }
}

async fn create0(_: &Factory, arg: &dyn CreateArg) -> Result<Box<dyn Action>, Rae> {
    let image = arg.config()["image"]
        .as_str()
        .ok_or(err!("010", "missing image"))?;
    let image = if image.contains(":") {
        image.into()
    } else {
        format!("{}:latest", image)
    };

    call(
        format!("images/create?fromImage={}", image).as_str(),
        Method::Post,
        None,
        1,
    )
    .await?;
    Ok(Box::new(Runner { image }))
}

async fn run0(runner: &Runner, arg: &dyn RunArg) -> Result<Value, Rae> {
    let cmd = arg.render_value(&arg.config()["cmd"]).unwrap_or(json!([]));
    let tail = arg
        .render_value(&arg.config()["tail"])?
        .as_u64()
        .unwrap_or(1) as usize;

    call(
        format!("containers/create?name={}", arg.id()).as_str(),
        Method::Post,
        Some(json!({ "Image": runner.image.as_str(),
                            "Cmd": cmd
        })),
        1,
    )
    .await?;
    trace!("create {}", arg.id());

    call(
        format!("containers/{}/start", arg.id()).as_str(),
        Method::Post,
        None,
        1,
    )
    .await?;
    trace!("start {}", arg.id());

    let log_content = call(
        format!(
            "containers/{}/logs?stdout=true&stderr=true&tail={}",
            arg.id(),
            tail
        )
        .as_str(),
        Method::Get,
        None,
        tail,
    )
    .await?;
    trace!("log {}", arg.id());

    call(
        format!("containers/{}?force=true", arg.id()).as_str(),
        Method::Delete,
        None,
        1,
    )
    .await?;
    trace!("remove {}", arg.id());

    Ok(from_str(log_content.join("").as_str())?)
}

async fn call(
    uri: &str,
    method: Method,
    data: Option<Value>,
    tail_size: usize,
) -> Result<Vec<String>, Rae> {
    let url = format!("http://localhost:2375/{}", uri);
    let url = Url::from_str(url.as_str()).or(rerr!("docker", format!("invalid url: {}", url)))?;
    let mut rb = RequestBuilder::new(method, url);
    rb = rb.header(
        HeaderName::from_str("Content-Type").unwrap(),
        HeaderValue::from_str("application/json")?,
    );
    if let Some(d) = data {
        rb = rb.body(d);
    }

    let mut res: Response = rb.send().await?;

    let mut tail: Vec<String> = vec![];
    let mut line = String::new();
    loop {
        line.clear();
        let size = res
            .read_line(&mut line)
            .await
            .or_else(|e| rcause!("docker", "read fail", e))?;
        if size > 0 {
            if res.content_type().is_some()
                && res.content_type().unwrap().to_string() == "application/octet-stream"
            {
                line = String::from_utf8_lossy(&line.as_bytes()[8..]).to_string();
            }

            trace!("{}", line.trim_end());
            if tail.len() >= tail_size {
                tail.pop();
            }
            tail.push(line.clone());
        } else {
            break;
        }
    }
    return if !res.status().is_success() {
        rerr!("docker", res.status().to_string())?
    } else {
        Ok(tail)
    };
}

struct Rae(chord::Error);

impl From<surf::Error> for Rae {
    fn from(err: surf::Error) -> Rae {
        Rae(err!("restapi", format!("{}", err.status())))
    }
}

impl From<chord::Error> for Rae {
    fn from(err: Error) -> Self {
        Rae(err)
    }
}

impl From<chord::value::Error> for Rae {
    fn from(err: chord::value::Error) -> Self {
        Rae(cause!("docker", "parse fail", err))
    }
}
