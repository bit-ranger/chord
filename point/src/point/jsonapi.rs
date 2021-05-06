use std::str::FromStr;

use surf::{Body, RequestBuilder, Response, Url};
use surf::http::headers::{HeaderName, HeaderValue};
use surf::http::Method;

use chord_common::{err, rerr};
use chord_common::error::Error;
use chord_common::point::{PointArg, PointValue, PointRunner, async_trait};
use chord_common::value::{Json, Map, Number};

struct Jsonapi {}

#[async_trait]
impl PointRunner for Jsonapi {

    async fn run(&self, arg: &dyn PointArg) -> PointValue {
        run(arg).await
    }
}

pub async fn create(_: &Json) -> Result<Box<dyn PointRunner>, Error>{
    Ok(Box::new(Jsonapi {}))
}


async fn run(arg: &dyn PointArg) -> PointValue{
    return run0(arg).await.map_err(|e| e.0);
}

async fn run0(arg: &dyn PointArg) -> std::result::Result<Json, Rae>{

    let url = arg.config()["url"].as_str()
        .map(|s|arg.render(s))
        .ok_or(err!("010", "missing url"))??;
    let url = Url::from_str(url.as_str()).or(rerr!("011", format!("invalid url: {}", url)))?;

    let method = arg.config()["method"].as_str()
        .map(|s|arg.render(s))
        .ok_or(err!("020", "missing method"))??;
    let method = Method::from_str(method.as_str()).or(rerr!("021", "invalid method"))?;

    let mut rb = RequestBuilder::new(method, url);
    rb = rb.header(HeaderName::from_str("Content-Type").unwrap(), HeaderValue::from_str("application/json")?);

    if let Some(header) = arg.config()["header"].as_object() {
        for (k, v) in header.iter() {
            let hn = HeaderName::from_string(arg.render(k)?)
                .or(rerr!("030", "invalid header name"))?;
            let hvt = arg.render(v.as_str().ok_or(err!("031", "invalid header value"))?)?;
            let hv = HeaderValue::from_str(hvt.as_str())
                .or(rerr!("031", "invalid header value"))?;
            rb = rb.header(hn, hv);
        }
    }

    if let Some(body) = arg.config()["body"].as_str(){
        rb = rb.body(Body::from_string(arg.render(body)?));
    }

    let mut res: Response = rb.send().await?;
    let mut res_data = Map::new();
    res_data.insert(String::from("status"), Json::Number(Number::from_str(res.status().to_string().as_str()).unwrap()));

    let mut header_data = Map::new();
    for header_name in res.header_names() {
        header_data.insert(header_name.to_string(), Json::String(res.header(header_name).unwrap().to_string()));
    }

    res_data.insert(String::from("header"), Json::Object(header_data));

    let body: Json = res.body_json().await?;
    res_data.insert(String::from("body"), body);
    return Ok(Json::Object(res_data))

}

struct Rae(chord_common::error::Error);


impl From<surf::Error> for Rae {
    fn from(err: surf::Error) -> Rae {
        Rae(err!("http", format!("{}", err.status())))
    }
}

impl From<chord_common::error::Error> for Rae {
    fn from(err: Error) -> Self {
        Rae(err)
    }
}





