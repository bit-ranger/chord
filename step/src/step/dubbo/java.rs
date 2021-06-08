use std::str::FromStr;

use async_std::io::BufReader;
use async_std::prelude::*;
use async_std::process::{Child, Command, Stdio};
use log::trace;
use surf::http::headers::{HeaderName, HeaderValue};
use surf::http::Method;
use surf::{Body, RequestBuilder, Response, Url};

use chord_common::error::Error;
use chord_common::step::{
    async_trait, CreateArg, RunArg, StepRunner, StepRunnerFactory, StepValue,
};
use chord_common::value::Json;
use chord_common::{err, rerr};

use serde::{Deserialize, Serialize};

pub struct Factory {
    registry_protocol: String,
    registry_address: String,
    port: usize,
    child: Child,
}

impl Factory {
    pub async fn new(config: Option<Json>) -> Result<Factory, Error> {
        if config.is_none() {
            return rerr!("010", "missing config");
        }
        let config = config.as_ref().unwrap();

        if config.is_null() {
            return rerr!("010", "missing config");
        }

        let jar_path = config["jar_path"]
            .as_str()
            .ok_or(err!("010", "missing jar_path"))?;

        let port = config["port"]
            .as_u64()
            .map(|p| p as usize)
            .ok_or(err!("010", "missing port"))?;

        let registry_protocol = config["registry"]["protocol"]
            .as_str()
            .unwrap_or("zookeeper")
            .to_owned();

        let registry_address = config["registry"]["address"]
            .as_str()
            .ok_or(err!("010", "missing registry_address"))?
            .to_owned();

        let mut child = Command::new("java")
            .arg("-jar")
            .arg(jar_path)
            .arg(format!("--server.port={}", port))
            .stdout(Stdio::piped())
            .spawn()?;

        let std_out = child.stdout.ok_or(err!("020", "missing stdout"))?;
        let std_out = BufReader::new(std_out);
        let mut lines = std_out.lines();
        loop {
            let line = lines.next().await;
            if line.is_none() {
                return rerr!("020", "failed to start dubbo-generic-gateway");
            }
            let line = line.unwrap()?;
            trace!("{}", line);
            if line == "----dubbo-generic-gateway-started----" {
                break;
            }
        }

        child.stdout = None;
        Ok(Factory {
            registry_protocol,
            registry_address,
            port,
            child,
        })
    }
}

#[async_trait]
impl StepRunnerFactory for Factory {
    async fn create(&self, _: &dyn CreateArg) -> Result<Box<dyn StepRunner>, Error> {
        Ok(Box::new(Runner {
            registry_protocol: self.registry_protocol.clone(),
            registry_address: self.registry_address.clone(),
            port: self.port,
        }))
    }
}

struct Runner {
    registry_protocol: String,
    registry_address: String,
    port: usize,
}

#[async_trait]
impl StepRunner for Runner {
    async fn run(&self, arg: &dyn RunArg) -> StepValue {
        let method_long = arg.config()["method"]
            .as_str()
            .ok_or(err!("010", "missing method"))?;
        let parts = method_long
            .split(&['#', '(', ',', ')'][..])
            .collect::<Vec<&str>>();
        if parts.len() < 2 {
            return rerr!("010", "invalid method");
        }

        let args = arg.config()["args"]
            .as_array()
            .ok_or(err!("010", "missing args"))?;

        let invoke = GenericInvoke {
            reference: Reference {
                application: "chord".to_owned(),
                registry_protocol: self.registry_protocol.clone(),
                registry_address: self.registry_address.clone(),
            },
            interface: parts[0].to_owned(),
            method: parts[1].to_owned(),
            parameter_types: parts[2..].iter().map(|p| p.to_owned().to_owned()).collect(),
            args: args.clone(),
        };

        let value = remote_invoke(self.port, invoke).await.map_err(|e| e.0)?;
        Ok(value)
    }
}

async fn remote_invoke(port: usize, remote_arg: GenericInvoke) -> Result<Json, Rae> {
    let rb = RequestBuilder::new(
        Method::Post,
        Url::from_str(format!("http://127.0.0.1:{}/invoke", port).as_str())
            .or(rerr!("021", "invalid url"))?,
    );
    let mut rb = rb.header(
        HeaderName::from_str("Content-Type").unwrap(),
        HeaderValue::from_str("application/json")?,
    );

    rb = rb.body(Body::from_json(&remote_arg)?);

    let mut res: Response = rb.send().await?;
    if !res.status().is_success() {
        return rerr!("dubbo", res.status().to_string())?;
    } else {
        let body: Json = res.body_json().await?;
        Ok(body)
    }
}

impl Drop for Factory {
    fn drop(&mut self) {
        let _ = self.child.kill();
    }
}

#[derive(Serialize, Deserialize)]
struct Reference {
    application: String,
    registry_protocol: String,
    registry_address: String,
}

#[derive(Serialize, Deserialize)]
struct GenericInvoke {
    reference: Reference,
    interface: String,
    method: String,
    parameter_types: Vec<String>,
    args: Vec<Json>,
}

struct Rae(chord_common::error::Error);

impl From<surf::Error> for Rae {
    fn from(err: surf::Error) -> Rae {
        Rae(err!("http", format!("{}", err.status())))
    }
}

impl From<serde_json::error::Error> for Rae {
    fn from(err: serde_json::error::Error) -> Rae {
        Rae(err!(
            "serde_json",
            format!("{}:{}", err.line(), err.column())
        ))
    }
}

impl From<chord_common::error::Error> for Rae {
    fn from(err: Error) -> Self {
        Rae(err)
    }
}
