use std::borrow::Borrow;
use std::str::FromStr;

use surf::http::headers::{HeaderName, HeaderValue};
use surf::http::Method;
use surf::{Body, RequestBuilder, Response, Url};

use chord::action::prelude::*;
use chord::value::{Map, Number};
use std::fmt::{Display, Formatter};

pub struct RestapiFactory {}

impl RestapiFactory {
    pub async fn new(_: Option<Value>) -> Result<RestapiFactory, Error> {
        Ok(RestapiFactory {})
    }
}

#[async_trait]
impl Factory for RestapiFactory {
    async fn create(&self, _: &dyn CreateArg) -> Result<Box<dyn Action>, Error> {
        Ok(Box::new(Restapi {}))
    }
}

struct Restapi {}

#[async_trait]
impl Action for Restapi {
    async fn run(&self, arg: &dyn RunArg) -> Result<Box<dyn Scope>, Error> {
        run(arg).await
    }

    async fn explain(&self, arg: &dyn RunArg) -> Result<Value, Error> {
        let args = arg.args(None)?;
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

async fn run(arg: &dyn RunArg) -> Result<Box<dyn Scope>, Error> {
    let value = run0(arg).await.map_err(|e| e.0)?;
    Ok(Box::new(value))
}

async fn run0(arg: &dyn RunArg) -> std::result::Result<Value, RestapiError> {
    let args = arg.args(None)?;

    let url = args["url"].as_str().ok_or(err!("100", "missing url"))?;
    let url = Url::from_str(url).or(Err(err!("101", format!("invalid url: {}", url))))?;

    let method = args["method"]
        .as_str()
        .ok_or(err!("102", "missing method"))?;
    let method = Method::from_str(method).or(Err(err!("103", "invalid method")))?;

    let mut rb = RequestBuilder::new(method, url);
    rb = rb.header(
        HeaderName::from_str("Content-Type").unwrap(),
        HeaderValue::from_str("application/json; charset=utf-8")?,
    );

    if let Some(header) = args["header"].as_object() {
        for (k, v) in header.iter() {
            let hn =
                HeaderName::from_string(k.clone()).or(Err(err!("104", "invalid header name")))?;
            let hvs: Vec<HeaderValue> = match v {
                Value::String(v) => {
                    vec![HeaderValue::from_str(v).or(Err(err!("105", "invalid header value")))?]
                }
                Value::Array(vs) => {
                    let mut vec = vec![];
                    for v in vs {
                        let v = HeaderValue::from_str(v.to_string().as_str())?;
                        vec.push(v)
                    }
                    vec
                }
                _ => Err(err!("106", "invalid header value"))?,
            };
            rb = rb.header(hn, hvs.as_slice());
        }
    }

    let body = args["body"].borrow();
    if !body.is_null() {
        rb = rb.body(Body::from(body.clone()));
    }

    let mut res: Response = rb.send().await?;
    let mut res_data = Map::new();
    res_data.insert(
        String::from("status"),
        Value::Number(Number::from_str(res.status().to_string().as_str()).unwrap()),
    );

    let mut header_data = Map::new();
    for (hn, hv) in res.iter() {
        header_data.insert(
            hn.to_string(),
            Value::Array(hv.iter().map(|v| Value::String(v.to_string())).collect()),
        );
    }
    res_data.insert(String::from("header"), Value::Object(header_data));

    let body: Value = res.body_json().await?;
    res_data.insert(String::from("body"), body);
    return Ok(Value::Object(res_data));
}

struct RestapiError(chord::Error);

impl From<surf::Error> for RestapiError {
    fn from(err: surf::Error) -> RestapiError {
        RestapiError(err!("107", format!("{}", err.status())))
    }
}

impl From<chord::Error> for RestapiError {
    fn from(err: Error) -> Self {
        RestapiError(err)
    }
}

impl From<chord::value::Error> for RestapiError {
    fn from(err: chord::value::Error) -> Self {
        RestapiError(err.into())
    }
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
