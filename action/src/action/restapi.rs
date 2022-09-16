use std::borrow::Borrow;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

use reqwest::header::{HeaderName, HeaderValue};
use reqwest::{Client, Method, Response, Url};

use chord_core::action::prelude::*;

use crate::err;

pub struct RestapiCreator {
    client: Client,
}

impl RestapiCreator {
    pub async fn new(_: Option<Value>) -> Result<RestapiCreator, Error> {
        let client = Client::new();
        Ok(RestapiCreator { client })
    }
}

#[async_trait]
impl Creator for RestapiCreator {
    async fn create(&self, _: &dyn Arg) -> Result<Box<dyn Action>, Error> {
        Ok(Box::new(RestapiAction {
            client: self.client.clone(),
        }))
    }
}

struct RestapiAction {
    client: Client,
}

#[async_trait]
impl Action for RestapiAction {
    async fn execute(&self, arg: &mut dyn Arg) -> Result<Box<dyn Scope>, Error> {
        run(self.client.clone(), arg).await
    }

    async fn explain(&self, arg: &dyn Arg) -> Result<Value, Error> {
        let args = arg.body()?;
        let url = args["url"].as_str().ok_or(err!("100", "missing url"))?;

        let url = Url::from_str(url).map_err(|_| err!("101", format!("invalid url: {}", url)))?;
        let method = args["method"]
            .as_str()
            .ok_or(err!("102", "missing method"))?
            .to_string();

        let mut headers = Vec::new();
        headers.push((
            "Content-Type".to_string(),
            "application/json; charset=utf-8".to_string(),
        ));
        if let Some(header) = args["header"].as_object() {
            for (k, v) in header.iter() {
                match v {
                    Value::String(v) => {
                        headers.push((k.clone(), v.clone()));
                    }
                    Value::Array(vs) => {
                        for v in vs {
                            headers.push((k.clone(), v.to_string()));
                        }
                    }
                    _ => Err(err!("106", "invalid header value"))?,
                };
            }
        };
        let body_raw = args["body"].borrow();
        let mut body = None;
        if !body_raw.is_null() {
            body = Some(body_raw.clone());
        };

        let curl = Curl {
            method,
            url,
            headers,
            body,
        };

        Ok(Value::String(curl.to_string()))
    }
}

async fn run(client: Client, arg: &dyn Arg) -> Result<Box<dyn Scope>, Error> {
    let value = run0(client, arg).await?;
    Ok(Box::new(value))
}

async fn run0(client: Client, arg: &dyn Arg) -> std::result::Result<Value, Error> {
    let args = arg.body()?;

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
        rb = rb.body(body.to_string());
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
        let body = body_str.parse().unwrap_or_else(|_| Value::String(body_str));
        res_data.insert(String::from("body"), body);
    };

    return Ok(Value::Object(res_data));
}

struct Curl {
    method: String,
    url: Url,
    headers: Vec<(String, String)>,
    body: Option<Value>,
}

impl Display for Curl {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        let mut curl = format!(
            r#"curl -X {} --location "{}" "#,
            self.method.to_uppercase(),
            self.url
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
