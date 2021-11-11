use std::str::FromStr;

use async_std::io::BufReader;
use async_std::process::{Child, ChildStdout, Command, Stdio};
use async_std::task::Builder;
use futures::io::Lines;
use futures::{AsyncBufReadExt, StreamExt};
use log::{debug, info, trace};
use surf::http::headers::{HeaderName, HeaderValue};
use surf::http::Method;
use surf::{Body, RequestBuilder, Response, Url};

use chord::action::prelude::*;
use chord::err;
use chord::value::{to_string, Deserialize, Serialize};
use chord::Error;

pub struct DubboFactory {
    registry_protocol: String,
    registry_address: String,
    port: usize,
    child: Child,
}

impl DubboFactory {
    pub async fn new(config: Option<Value>) -> Result<DubboFactory, Box<dyn Error>> {
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
            .ok_or(err!("102", "missing dubbo.gateway.registry.address"))?
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
        let port: usize = port
            .get(1)
            .ok_or(err!("106", "missing dubbo.gateway.args.--server.port"))?
            .parse()?;

        let mut command = Command::new("java");
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
            let line = lines.next().await;
            if line.is_none() {
                return Err(err!("108", "failed to start dubbo-generic-gateway"));
            }
            let line = line.unwrap()?;
            log_line(&line).await;
            if line == "----dubbo-generic-gateway-started----" {
                break;
            }
        }

        let _ = Builder::new()
            .name("dubbo-gateway-output".into())
            .spawn(loop_out(lines));

        child.stdout = None;
        Ok(DubboFactory {
            registry_protocol,
            registry_address,
            port,
            child,
        })
    }
}

#[async_trait]
impl Factory for DubboFactory {
    async fn create(&self, _: &dyn CreateArg) -> Result<Box<dyn Action>, Box<dyn Error>> {
        Ok(Box::new(Dubbo {
            registry_protocol: self.registry_protocol.clone(),
            registry_address: self.registry_address.clone(),
            port: self.port,
        }))
    }
}

struct Dubbo {
    registry_protocol: String,
    registry_address: String,
    port: usize,
}

#[async_trait]
impl Action for Dubbo {
    async fn run(&self, arg: &dyn RunArg) -> Result<Box<dyn Scope>, Box<dyn Error>> {
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
                timeout: arg.timeout().as_millis() as usize,
            },

            method: parts[1].to_owned(),
            arg_types: parts[2..].iter().map(|p| p.to_owned().to_owned()).collect(),
            args: args.clone(),
        };

        let invoke_str = to_string(&invoke)?;
        trace!("invoke request {}", invoke_str);
        let value = remote_invoke(self.port, invoke).await.map_err(|e| e.0)?;
        trace!("invoke response {}, {}", invoke_str, &value);
        let value = &value;
        if value["success"].as_bool().unwrap_or(false) {
            return Ok(Box::new(value["data"].clone()));
        }

        return Err(err!(
            "113",
            format!("{}::{}", value["code"], value["message"])
        ));
    }
}

async fn remote_invoke(port: usize, remote_arg: GenericInvoke) -> Result<Value, DubboError> {
    let rb = RequestBuilder::new(
        Method::Post,
        Url::from_str(format!("http://127.0.0.1:{}/api/dubbo/generic/invoke", port).as_str())
            .or(Err(err!("114", "invalid url")))?,
    );
    let mut rb = rb.header(
        HeaderName::from_str("Content-Type").unwrap(),
        HeaderValue::from_str("application/json")?,
    );

    rb = rb.body(Body::from_json(&remote_arg)?);

    let mut res: Response = rb.send().await?;
    if !res.status().is_success() {
        return Err(err!("115", res.status().to_string()))?;
    } else {
        let body: Value = res.body_json().await?;
        Ok(body)
    }
}

async fn loop_out(mut lines: Lines<BufReader<ChildStdout>>) {
    loop {
        let line = lines.next().await;
        if line.is_none() {
            break;
        }
        if let Ok(line) = line.unwrap() {
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

impl Drop for DubboFactory {
    fn drop(&mut self) {
        let _ = self.child.kill();
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

struct DubboError(chord::Error);

impl From<surf::Error> for DubboError {
    fn from(err: surf::Error) -> DubboError {
        DubboError(err!("116", format!("{}", err.status())))
    }
}

impl From<chord::value::Error> for DubboError {
    fn from(err: chord::value::Error) -> DubboError {
        DubboError(err!("117", format!("{}:{}", err.line(), err.column())))
    }
}

impl From<chord::Error> for DubboError {
    fn from(err: Error) -> Self {
        DubboError(err)
    }
}
