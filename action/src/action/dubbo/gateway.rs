use std::borrow::{Borrow, BorrowMut};
use std::process::Stdio;
use std::str::FromStr;

use log::{debug, info, trace};
use reqwest::{Body, Client, Method, Response, Url};
use reqwest::header::{HeaderName, HeaderValue};

use chord_core::action::prelude::*;
use chord_core::future::io::{AsyncBufReadExt, BufReader, Lines};
use chord_core::future::process::{Child, ChildStdout, Command};
use chord_core::future::sync::RwLock;
use chord_core::future::task::spawn;
use chord_core::value::{Deserialize, Serialize, to_string};

use crate::err;

pub struct DubboCreator {
    actual: RwLock<Option<DubboCreatorActual>>,
    registry_protocol: String,
    registry_address: String,
    gateway_lib: String,
    gateway_args: Vec<String>,
    gateway_port: usize,
}

impl DubboCreator {
    pub async fn new(config: Option<Value>) -> Result<DubboCreator, Error> {
        if config.is_none() {
            return Err(err!("100", "missing action.dubbo"));
        }
        let config = config.as_ref().unwrap();

        if config.is_null() {
            return Err(err!("101", "missing action.dubbo"));
        }

        let registry_protocol = config["gateway"]["registry"]["protocol"]
            .as_str()
            .unwrap_or("zookeeper")
            .to_owned();

        let registry_address = config["gateway"]["registry"]["address"]
            .as_str()
            .unwrap_or("127.0.0.1:2181")
            .to_owned();

        let gateway_lib = config["gateway"]["lib"]
            .as_str()
            .ok_or(err!("103", "missing dubbo.gateway.lib"))?
            .to_owned();

        let gateway_args: Vec<String> = config["gateway"]["args"]
            .as_array()
            .map(|a| {
                a.iter()
                    .map(|a| {
                        if a.is_string() {
                            a.as_str().unwrap().to_owned()
                        } else {
                            a.to_string()
                        }
                    })
                    .collect()
            })
            .unwrap_or(vec![
                "-Ddubbo.application.qos.enable=false".to_string(),
                "--server.port=8085".to_string(),
            ]);

        let port = gateway_args
            .iter()
            .filter(|a| a.trim().starts_with("--server.port="))
            .last()
            .ok_or(err!("105", "missing dubbo.gateway.args.--server.port"))?;
        let port: Vec<&str> = port.split("=").collect();
        let gateway_port: usize = port
            .get(1)
            .ok_or(err!("106", "missing dubbo.gateway.args.--server.port"))?
            .parse()?;

        Ok(DubboCreator {
            actual: RwLock::new(None),
            registry_protocol,
            registry_address,
            gateway_lib,
            gateway_args,
            gateway_port,
        })
    }
}

#[async_trait]
impl Creator for DubboCreator {
    async fn create(&self, chord: &dyn Chord, arg: &dyn Arg) -> Result<Box<dyn Action>, Error> {
        let creator = self.actual.read().await;
        let creator_ref = creator.borrow();
        if creator_ref.is_some() {
            return creator_ref.as_ref().unwrap().create(chord, arg).await;
        } else {
            drop(creator);
            let mut guard = self.actual.write().await;
            let guard = guard.borrow_mut();
            let new_creator = DubboCreatorActual::new(
                self.registry_protocol.clone(),
                self.registry_address.clone(),
                self.gateway_lib.clone(),
                self.gateway_args.clone(),
                self.gateway_port,
            )
            .await?;
            let action = new_creator.create(chord, arg).await?;
            guard.replace(new_creator);
            return Ok(action);
        }
    }
}

struct DubboCreatorActual {
    registry_protocol: String,
    registry_address: String,
    gateway_port: usize,
    child: Child,
    client: Client,
}

impl DubboCreatorActual {
    pub async fn new(
        registry_protocol: String,
        registry_address: String,
        gateway_lib: String,
        gateway_args: Vec<String>,
        gateway_port: usize,
    ) -> Result<DubboCreatorActual, Error> {
        let mut command = Command::new("java");
        command.kill_on_drop(true);
        command.arg("-jar").arg(gateway_lib);
        for arg in gateway_args {
            command.arg(arg);
        }

        trace!("command {:?}", command);

        let mut child = command
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        let std_out = child.stdout.ok_or(err!("107", "missing stdout"))?;
        let std_out = BufReader::new(std_out);
        let mut lines = std_out.lines();
        loop {
            let line = lines.next_line().await?;
            if line.is_none() {
                return Err(err!("108", "failed to start dubbo-generic-gateway"));
            }
            let line = line.unwrap();
            log_line(&line).await;
            if line == "----dubbo-generic-gateway-started----" {
                break;
            }
        }

        let _ = spawn(loop_out(lines));

        child.stdout = None;
        let client = Client::new();
        Ok(DubboCreatorActual {
            registry_protocol,
            registry_address,
            gateway_port,
            child,
            client,
        })
    }
}

#[async_trait]
impl Creator for DubboCreatorActual {
    async fn create(&self, _chord: &dyn Chord, _arg: &dyn Arg) -> Result<Box<dyn Action>, Error> {
        Ok(Box::new(Dubbo {
            registry_protocol: self.registry_protocol.clone(),
            registry_address: self.registry_address.clone(),
            gateway_port: self.gateway_port,
            client: self.client.clone(),
        }))
    }
}

struct Dubbo {
    registry_protocol: String,
    registry_address: String,
    gateway_port: usize,
    client: Client,
}

#[async_trait]
impl Action for Dubbo {
    async fn execute(
        &self,
        _chord: &dyn Chord,
        arg: &mut dyn Arg,
    ) -> Result<Asset, Error> {
        let args = arg.args()?;
        let method_long = args["method"]
            .as_str()
            .ok_or(err!("109", "missing method"))?;
        let parts = method_long
            .split(&['#', '(', ',', ')'][..])
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect::<Vec<&str>>();
        if parts.len() < 2 {
            return Err(err!("110", "invalid method"));
        }

        let args = args["args"]
            .as_array()
            .ok_or(err!("111", "args must be array"))?;

        let invoke = GenericInvoke {
            reference: Reference {
                application: Application {
                    name: "chord".to_owned(),
                },
                registry: Registry {
                    protocol: self.registry_protocol.clone(),
                    address: self.registry_address.clone(),
                },
                interface: parts[0].to_owned(),
                timeout: 60000,
            },

            method: parts[1].to_owned(),
            arg_types: parts[2..].iter().map(|p| p.to_owned().to_owned()).collect(),
            args: args.clone(),
        };

        let invoke_str = to_string(&invoke)?;
        trace!("invoke request {}", invoke_str);
        let value = remote_invoke(self.client.clone(), self.gateway_port, invoke).await?;
        trace!("invoke response {}, {}", invoke_str, &value);
        let value = &value;
        if value["success"].as_bool().unwrap_or(false) {
            return Ok(Asset::Value(value["data"].clone()));
        }

        return Err(err!(
            "113",
            format!("{}::{}", value["code"], value["message"])
        ));
    }
}

async fn remote_invoke(
    client: Client,
    port: usize,
    remote_arg: GenericInvoke,
) -> Result<Value, Error> {
    let rb = client.request(
        Method::POST,
        Url::from_str(format!("http://127.0.0.1:{}/api/dubbo/generic/invoke", port).as_str())
            .or(Err(err!("114", "invalid url")))?,
    );
    let mut rb = rb.header(
        HeaderName::from_str("Content-Type").unwrap(),
        HeaderValue::from_str("application/json")?,
    );

    rb = rb.body(Body::from(to_string(&remote_arg)?));

    let res: Response = rb.send().await?;
    if !res.status().is_success() {
        return Err(err!("115", res.status().to_string()))?;
    } else {
        let body: String = res.text().await?;
        let body: Value = body.parse()?;
        Ok(body)
    }
}

async fn loop_out(mut lines: Lines<BufReader<ChildStdout>>) {
    loop {
        let line = lines.next_line().await.unwrap();
        if line.is_none() {
            break;
        }
        if let Some(line) = line {
            log_line(&line).await;
        } else {
            break;
        }
    }
}

async fn log_line(line: &str) {
    if line.len() > 30 {
        match &line[24..29] {
            "ERROR" => info!("{}", line),
            " WARN" => debug!("{}", line),
            " INFO" => trace!("{}", line),
            "DEBUG" => trace!("{}", line),
            _ => trace!("{}", line),
        }
    } else {
        trace!("{}", line)
    }
}

impl Drop for DubboCreatorActual {
    fn drop(&mut self) {
        trace!(
            "kill [{}] dubbo generic gateway",
            self.child.id().unwrap_or(0)
        );
    }
}

#[derive(Serialize, Deserialize)]
struct GenericInvoke {
    reference: Reference,
    method: String,
    arg_types: Vec<String>,
    args: Vec<Value>,
}

#[derive(Serialize, Deserialize)]
struct Reference {
    interface: String,
    application: Application,
    registry: Registry,
    timeout: usize,
}

#[derive(Serialize, Deserialize)]
struct Application {
    name: String,
}

#[derive(Serialize, Deserialize)]
struct Registry {
    protocol: String,
    address: String,
}
