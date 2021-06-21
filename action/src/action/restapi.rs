use std::borrow::Borrow;
use std::str::FromStr;

use surf::http::headers::{HeaderName, HeaderValue};
use surf::http::Method;
use surf::{Body, RequestBuilder, Response, Url};

use chord::step::{async_trait, Action, ActionFactory, ActionValue, CreateArg, RunArg};
use chord::value::{Map, Number, Value};
use chord::Error;
use chord::{err, rerr};

pub struct Factory {}

impl Factory {
    pub async fn new(_: Option<Value>) -> Result<Factory, Error> {
        Ok(Factory {})
    }
}

#[async_trait]
impl ActionFactory for Factory {
    async fn create(&self, _: &dyn CreateArg) -> Result<Box<dyn Action>, Error> {
        Ok(Box::new(Runner {}))
    }
}

struct Runner {}

#[async_trait]
impl Action for Runner {
    async fn run(&self, arg: &dyn RunArg) -> ActionValue {
        run(arg).await
    }
}

async fn run(arg: &dyn RunArg) -> ActionValue {
    return run0(arg).await.map_err(|e| e.0);
}

async fn run0(arg: &dyn RunArg) -> std::result::Result<Value, Rae> {
    let url = arg.config()["url"]
        .as_str()
        .map(|s| arg.render_str(s))
        .ok_or(err!("010", "missing url"))??;
    let url = Url::from_str(url.as_str()).or(rerr!("011", format!("invalid url: {}", url)))?;

    let method = arg.config()["method"]
        .as_str()
        .map(|s| arg.render_str(s))
        .ok_or(err!("020", "missing method"))??;
    let method = Method::from_str(method.as_str()).or(rerr!("021", "invalid method"))?;

    let mut rb = RequestBuilder::new(method, url);
    rb = rb.header(
        HeaderName::from_str("Content-Type").unwrap(),
        HeaderValue::from_str("application/json")?,
    );

    if let Some(header) = arg.config()["header"].as_object() {
        for (k, v) in header.iter() {
            let hn = HeaderName::from_string(arg.render_str(k)?)
                .or(rerr!("030", "invalid header name"))?;
            let hvt = arg.render_str(v.as_str().ok_or(err!("031", "invalid header value"))?)?;
            let hv =
                HeaderValue::from_str(hvt.as_str()).or(rerr!("031", "invalid header value"))?;
            rb = rb.header(hn, hv);
        }
    }

    let body = arg.config()["body"].borrow();
    if !body.is_null() {
        let body = arg.render_value(body)?;
        rb = rb.body(Body::from(body));
    }

    let mut res: Response = rb.send().await?;
    let mut res_data = Map::new();
    res_data.insert(
        String::from("status"),
        Value::Number(Number::from_str(res.status().to_string().as_str()).unwrap()),
    );

    let mut header_data = Map::new();
    for header_name in res.header_names() {
        header_data.insert(
            header_name.to_string(),
            Value::String(res.header(header_name).unwrap().to_string()),
        );
    }

    res_data.insert(String::from("header"), Value::Object(header_data));

    let body: Value = res.body_json().await?;
    res_data.insert(String::from("body"), body);
    return Ok(Value::Object(res_data));
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
