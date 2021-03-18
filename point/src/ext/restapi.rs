use std::str::FromStr;

use surf::{Body, RequestBuilder, Response, Url};
use surf::http::headers::{HeaderName, HeaderValue};
use surf::http::Method;

use common::point::PointArg;
use common::value::{Json, Map, Number};

use crate::err;
use crate::model::{Error, PointValue};

pub async fn run(context: &dyn PointArg) -> PointValue{
    let url = match context.config_rendered(vec!["url"]) {
        Some(url) => url,
        None => return err!("010", "missing url")
    };

    let url = match Url::from_str(url.as_str()) {
        Ok(url) => url,
        Err(_) => return err!("011", format!("invalid url: {}", url).as_str())
    };

    let method = match context.config_rendered(vec!["method"]) {
        Some(method) => method,
        None => return err!("020", "missing method")
    };

    let method = match Method::from_str(method.as_str()) {
        Ok(method) => method,
        Err(_) => return err!("021", "invalid method")
    };


    let mut rb = RequestBuilder::new(method, url);

    if let Some(header) = context.config()["header"].as_object() {
        for (k, v) in header.iter() {
            let hn = HeaderName::from_str(context.render(k)?.as_str());
            if hn.is_err() {
                return err!("030", "invalid header name");
            }
            let hv = HeaderValue::from_str(context.render(v.as_str().unwrap())?.as_str());
            if hv.is_err() {
                return err!("031", "invalid header value");
            }
            rb = rb.header(hn.unwrap(), hv.unwrap());
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





impl From<surf::Error> for Error {
    fn from(err: surf::Error) -> Error {
        Error::new("http", format!("{}", err.status()).as_str())
    }
}



