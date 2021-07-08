use std::borrow::Borrow;
use std::str::FromStr;

use surf::http::headers::{HeaderName, HeaderValue};
use surf::http::Method;
use surf::{Body, RequestBuilder, Response, Url};

use chord::action::prelude::*;
use chord::value::{from_str, Map, Number};

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
}

async fn run(arg: &dyn RunArg) -> Result<Box<dyn Scope>, Error> {
    let value = run0(arg).await.map_err(|e| e.0)?;
    Ok(Box::new(value))
}

async fn run0(arg: &dyn RunArg) -> std::result::Result<Value, RestapiError> {
    let args = arg.render_value(arg.args())?;

    let url = args["url"].as_str().ok_or(err!("010", "missing url"))?;
    let url = Url::from_str(url).or(Err(err!("011", format!("invalid url: {}", url))))?;

    let method = args["method"]
        .as_str()
        .ok_or(err!("020", "missing method"))?;
    let method = Method::from_str(method).or(Err(err!("021", "invalid method")))?;

    let mut rb = RequestBuilder::new(method, url);
    rb = rb.header(
        HeaderName::from_str("Content-Type").unwrap(),
        HeaderValue::from_str("application/json")?,
    );

    if let Some(header) = args["header"].as_object() {
        for (k, v) in header.iter() {
            let hn =
                HeaderName::from_string(k.clone()).or(Err(err!("030", "invalid header name")))?;
            let hvs: Vec<HeaderValue> = match v {
                Value::String(v) => {
                    vec![HeaderValue::from_str(v).or(Err(err!("031", "invalid header value")))?]
                }
                Value::Array(vs) => {
                    let mut vec = vec![];
                    for v in vs {
                        let v = HeaderValue::from_str(v.to_string().as_str())?;
                        vec.push(v)
                    }
                    vec
                }
                _ => Err(err!("031", "invalid header value"))?,
            };
            rb = rb.header(hn, hvs.as_slice());
        }
    }

    let body = args["body"].borrow();
    if !body.is_null() {
        match body {
            Value::String(txt) => {
                let body: Value = from_str(txt.as_str())?;
                rb = rb.body(Body::from(body));
            }
            _ => {
                rb = rb.body(Body::from(body.clone()));
            }
        }
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
        RestapiError(err!("restapi", format!("{}", err.status())))
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
