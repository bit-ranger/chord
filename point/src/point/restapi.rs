use std::str::FromStr;

use surf::{Body, RequestBuilder, Response, Url};
use surf::http::headers::{HeaderName, HeaderValue};
use surf::http::Method;

use chord_common::point::PointArg;
use chord_common::value::{Json, Map, Number};

use crate::{err,perr};
use crate::model::{PointError, PointValue};

pub async fn run(context: &dyn PointArg) -> PointValue{
    let url = context.config_rendered(vec!["url"]).ok_or(perr!("010", "missing url"))?;
    let url = Url::from_str(url.as_str()).or(err!("011", format!("invalid url: {}", url)))?;

    let method = context.config_rendered(vec!["method"]).ok_or(perr!("020", "missing method"))?;
    let method = Method::from_str(method.as_str()).or(err!("021", "invalid method"))?;

    let mut rb = RequestBuilder::new(method, url);
    rb = rb.header(HeaderName::from_str("Content-Type").unwrap(), HeaderValue::from_str("application/json").unwrap());

    if let Some(header) = context.config()["header"].as_object() {
        for (k, v) in header.iter() {
            let hn = HeaderName::from_string(context.render(k)?)
                .or(err!("030", "invalid header name"))?;
            let hvt = context.render(v.as_str().ok_or(perr!("031", "invalid header value"))?)?;
            let hv = HeaderValue::from_str(hvt.as_str())
                .or(err!("031", "invalid header value"))?;
            rb = rb.header(hn, hv);
        }
    }

    if let Some(body) = context.config_rendered(vec!["body"]){
        rb = rb.body(Body::from_string(body));
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





impl From<surf::Error> for PointError {
    fn from(err: surf::Error) -> PointError {
        PointError::new("http", format!("{}", err.status()))
    }
}



