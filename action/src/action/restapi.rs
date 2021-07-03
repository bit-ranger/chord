use std::borrow::Borrow;
use std::str::FromStr;

use surf::http::headers::{HeaderName, HeaderValue};
use surf::http::Method;
use surf::{Body, RequestBuilder, Response, Url};

use chord::action::prelude::*;
use chord::value::{Map, Number};

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
    let url = Url::from_str(url).or(rerr!("011", format!("invalid url: {}", url)))?;

    let method = args["method"]
        .as_str()
        .ok_or(err!("020", "missing method"))?;
    let method = Method::from_str(method).or(rerr!("021", "invalid method"))?;

    let mut rb = RequestBuilder::new(method, url);
    rb = rb.header(
        HeaderName::from_str("Content-Type").unwrap(),
        HeaderValue::from_str("application/json")?,
    );

    if let Some(header) = args["header"].as_object() {
        for (k, v) in header.iter() {
            let hn = HeaderName::from_string(k.clone()).or(rerr!("030", "invalid header name"))?;
            let hvt = v.as_str().ok_or(err!("031", "invalid header value"))?;
            let hv = HeaderValue::from_str(hvt).or(rerr!("031", "invalid header value"))?;
            rb = rb.header(hn, hv);
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
        header_data.insert(hn.to_string(), Value::String(hv.to_string()));
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
