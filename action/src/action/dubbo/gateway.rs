use std::str::FromStr;

use async_std::io::BufReader;
use async_std::prelude::*;
use async_std::process::{Child, ChildStdout, Command, Stdio};
use async_std::task::Builder;
use log::{debug, info, trace};
use surf::http::headers::{HeaderName, HeaderValue};
use surf::http::Method;
use surf::{Body, RequestBuilder, Response, Url};

use chord::action::prelude::*;
use chord::err;
use chord::value::{from_str, to_string, Deserialize, Serialize};
use chord::Error;

pub struct DubboFactory {
    registry_protocol: String,
    registry_address: String,
    port: usize,
    child: Child,
}

impl DubboFactory {
    pub async fn new(config: Option<Value>) -> Result<DubboFactory, Error> {
        if config.is_none() {
            return Err(err!("dubbo", "missing dubbo.config"));
        }
        let config = config.as_ref().unwrap();

        if config.is_null() {
            return Err(err!("dubbo", "missing dubbo.config"));
        }

        let registry_protocol = config["gateway"]["registry"]["protocol"]
            .as_str()
            .unwrap_or("zookeeper")
            .to_owned();

        let registry_address = config["gateway"]["registry"]["address"]
            .as_str()
            .ok_or(err!("dubbo", "missing dubbo.gateway.registry.address"))?
            .to_owned();

        let gateway_args = config["gateway"]["args"]
            .as_array()
            .ok_or(err!("dubbo", "missing dubbo.gateway.args"))?;
        let gateway_args: Vec<String> = gateway_args
            .iter()
            .map(|a| {
                if a.is_string() {
                    a.as_str().unwrap().to_owned()
                } else {
                    a.to_string()
                }
            })
            .collect();

        let _ = gateway_args
            .iter()
            .filter(|a| a.trim() == "-jar")
            .last()
            .ok_or(err!("dubbo", "missing dubbo.gateway.args.-jar"))?;

        let port = gateway_args
            .iter()
            .filter(|a| a.trim().starts_with("--server.port="))
            .last()
            .ok_or(err!("dubbo", "missing dubbo.gateway.args.--server.port"))?;
        let port: Vec<&str> = port.split("=").collect();
        let port: usize = port
            .get(1)
            .ok_or(err!("dubbo", "missing dubbo.gateway.args.--server.port"))?
            .parse()?;

        let mut command = Command::new("java");

        for arg in gateway_args {
            command.arg(arg);
        }

        trace!("command {:?}", command);

        let mut child = command
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        let std_out = child.stdout.ok_or(err!("dubbo", "missing stdout"))?;
        let mut std_out = BufReader::new(std_out);
        let mut lines = std_out.by_ref().lines();
        loop {
            let line = lines.next().await;
            if line.is_none() {
                return Err(err!("dubbo", "failed to start dubbo-generic-gateway"));
            }
            let line = line.unwrap()?;
            log_line(&line).await;
            if line == "----dubbo-generic-gateway-started----" {
                break;
            }
        }

        let _ = Builder::new()
            .name("dubbo-gateway-output".into())
            .spawn(loop_out(std_out));

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
    async fn create(&self, _: &dyn CreateArg) -> Result<Box<dyn Action>, Error> {
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
    async fn run(&self, run_arg: &dyn RunArg) -> Result<Box<dyn Scope>, Error> {
        let method_long = run_arg.args()["method"]
            .as_str()
            .ok_or(err!("010", "missing method"))?;
        let parts = method_long
            .split(&['#', '(', ',', ')'][..])
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect::<Vec<&str>>();
        if parts.len() < 2 {
            return Err(err!("010", "invalid method"));
        }

        let args_raw = &run_arg.args()["args"];
        let args = match args_raw {
            Value::String(txt) => {
                let txt_render = run_arg.render_str(txt.as_str())?;
                if &run_arg.args()["content_type"] == "json" {
                    let args_render = from_str(txt_render.as_str())?;
                    match args_render {
                        Value::Array(aw_vec) => aw_vec,
                        _ => vec![args_render],
                    }
                } else {
                    vec![Value::String(txt_render)]
                }
            }
            _ => {
                let args_render = run_arg.render_value(args_raw)?;
                match args_render {
                    Value::Array(aw_vec) => aw_vec,
                    _ => vec![args_render],
                }
            }
        };

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
            },

            method: parts[1].to_owned(),
            arg_types: parts[2..].iter().map(|p| p.to_owned().to_owned()).collect(),
            args,
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
            "dubbo",
            format!("{}::{}", value["code"], value["message"])
        ));
    }
}

async fn remote_invoke(port: usize, remote_arg: GenericInvoke) -> Result<Value, DubboError> {
    let rb = RequestBuilder::new(
        Method::Post,
        Url::from_str(format!("http://127.0.0.1:{}/api/dubbo/generic/invoke", port).as_str())
            .or(Err(err!("021", "invalid url")))?,
    );
    let mut rb = rb.header(
        HeaderName::from_str("Content-Type").unwrap(),
        HeaderValue::from_str("application/json")?,
    );

    rb = rb.body(Body::from_json(&remote_arg)?);

    let mut res: Response = rb.send().await?;
    if !res.status().is_success() {
        return Err(err!("dubbo", res.status().to_string()))?;
    } else {
        let body: Value = res.body_json().await?;
        Ok(body)
    }
}

async fn loop_out(mut std_out: BufReader<ChildStdout>) {
    let mut lines = std_out.by_ref().lines();
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
        DubboError(err!("dubbo", format!("{}", err.status())))
    }
}

impl From<chord::value::Error> for DubboError {
    fn from(err: chord::value::Error) -> DubboError {
        DubboError(err!("dubbo", format!("{}:{}", err.line(), err.column())))
    }
}

impl From<chord::Error> for DubboError {
    fn from(err: Error) -> Self {
        DubboError(err)
    }
}
