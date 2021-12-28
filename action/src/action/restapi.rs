use std::borrow::Borrow;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

use chord_core::action::prelude::*;

use crate::err;
use reqwest::header::{HeaderName, HeaderValue};
use reqwest::{Body, Client, Method, Response, Url};

pub struct RestapiFactory {
    client: Client,
}

impl RestapiFactory {
    pub async fn new(_: Option<Value>) -> Result<RestapiFactory, Error> {
        let client = Client::new();
        Ok(RestapiFactory { client })
    }
}

#[async_trait]
impl Factory for RestapiFactory {
    async fn create(&self, _: &dyn CreateArg) -> Result<Box<dyn Action>, Error> {
        Ok(Box::new(Restapi {
            client: self.client.clone(),
        }))
    }
}

struct Restapi {
    client: Client,
}

#[async_trait]
impl Action for Restapi {
    async fn run(&self, arg: &dyn RunArg) -> Result<Box<dyn Scope>, Error> {
        run(self.client.clone(), arg).await
    }

    async fn explain(&self, arg: &dyn RunArg) -> Result<Value, Error> {
        let args = arg.args()?;
        let mut curl = Curl::default();
        let url = args["url"].as_str().ok_or(err!("100", "missing url"))?;
        curl.url = url.to_string();
        let method = args["method"]
            .as_str()
            .ok_or(err!("102", "missing method"))?;
        curl.method = method.to_string();
        curl.headers.push((
            "Content-Type".to_string(),
            "application/json; charset=utf-8".to_string(),
        ));
        if let Some(header) = args["header"].as_object() {
            for (k, v) in header.iter() {
                match v {
                    Value::String(v) => {
                        curl.headers.push((k.clone(), v.clone()));
                    }
                    Value::Array(vs) => {
                        for v in vs {
                            curl.headers.push((k.clone(), v.to_string()));
                        }
                    }
                    _ => Err(err!("106", "invalid header value"))?,
                };
            }
        };
        let body = args["body"].borrow();
        if !body.is_null() {
            curl.body = Some(body.clone());
        }
        Ok(Value::String(curl.to_string()))
    }
}

async fn run(client: Client, arg: &dyn RunArg) -> Result<Box<dyn Scope>, Error> {
    let value = run0(client, arg).await?;
    Ok(Box::new(value))
}

async fn run0(client: Client, arg: &dyn RunArg) -> std::result::Result<Value, Error> {
    let args = arg.args()?;

    let url = args["url"].as_str().ok_or(err!("100", "missing url"))?;
    let url = Url::from_str(url).or(Err(err!("101", format!("invalid url: {}", url))))?;

    let method = args["method"]
        .as_str()
        .ok_or(err!("102", "missing method"))?;
    let method = Method::from_str(method).or(Err(err!("103", "invalid method")))?;

    let mut rb = client.request(method, url);
    rb = rb.header(
        HeaderName::from_str("Content-Type").unwrap(),
        HeaderValue::from_str("application/json; charset=utf-8")?,
    );

    if let Some(header) = args["header"].as_object() {
        for (k, v) in header.iter() {
            let hn = HeaderName::from_str(k).or(Err(err!("104", "invalid header name")))?;
            match v {
                Value::String(v) => {
                    let hv =
                        HeaderValue::from_str(v).or(Err(err!("105", "invalid header value")))?;
                    rb = rb.header(hn, hv);
                }
                Value::Array(vs) => {
                    for v in vs {
                        let hv = HeaderValue::from_str(v.to_string().as_str())?;
                        rb = rb.header(hn.clone(), hv);
                    }
                }
                _ => Err(err!("106", "invalid header value"))?,
            };
        }
    }

    let body = args["body"].borrow();
    if !body.is_null() {
        rb = rb.body(body.clone());
    }

    let res: Response = rb.send().await?;
    let mut res_data = Map::new();
    res_data.insert(
        String::from("status"),
        Value::Number(Number::from(res.status().as_u16())),
    );

    let mut header_data = Map::new();
    for (hn, hv) in res.headers() {
        header_data.insert(hn.to_string(), Value::String(hv.to_str()?.to_string()));
    }
    res_data.insert(String::from("header"), Value::Object(header_data));

    let body_str = res.text().await?;
    if !body_str.is_empty() {
        let body = body_str.parse()?;
        res_data.insert(String::from("body"), body);
    };

    return Ok(Value::Object(res_data));
}

#[derive(Default)]
struct Curl {
    method: String,
    url: String,
    headers: Vec<(String, String)>,
    body: Option<Value>,
}

impl Display for Curl {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        let mut curl = format!(
            r#"curl -X {} --location "{}" "#,
            self.method.to_uppercase(),
            self.url.escape_debug()
        );
        for (k, v) in &self.headers {
            curl.push_str(format!(r#"-H "{}:{}" "#, k.escape_debug(), v.escape_debug()).as_str());
        }
        if let Some(body) = &self.body {
            curl.push_str(format!(r#"-d"{}""#, body.to_string().escape_debug()).as_str())
        }
        f.write_str(curl.as_str())?;
        Ok(())
    }
}
