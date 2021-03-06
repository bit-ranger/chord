use crate::model::{Error, Json, PointResult, PointContext};
use surf::{RequestBuilder, Response, Url};
use surf::http::Method;
use std::str::FromStr;
use std::collections::HashMap;
use serde_json::{Value, Map, Number};
use surf::http::headers::{HeaderName, HeaderValue};
use futures::TryFutureExt;
use std::borrow::BorrowMut;

pub async fn run(context: &dyn PointContext) -> PointResult{
    let url = context.get_config_rendered(vec!["url"]).unwrap();
    println!("url {}", url);

    let method = context.get_config_rendered(vec!["method"]).unwrap();
    let mut rb = RequestBuilder::new(Method::from_str(method.as_str()).unwrap(),
                                     Url::from_str(url.as_str()).unwrap());

    if let Some(header) = context.get_config()["header"].as_object() {
        for (k,v) in header.iter() {
            rb = rb.header(HeaderName::from_str(context.render(k).as_str()).unwrap(),
                      HeaderValue::from_str(context.render(v.as_str().unwrap()).as_str()).unwrap());
        }
    }

    let mut response: surf::Result<Response> = rb.send().await;
    match response {
        Ok(mut res) => {
            let mut res_data = Map::<String, Json>::new();
            res_data.insert(String::from("status"), Json::String(String::from(res.status().to_string())));

            let mut header_data:Map<String,Json> = Map::new();
            for header_name in res.header_names() {
                header_data.insert(header_name.to_string(), Json::String(res.header(header_name).unwrap().to_string()));
            }

            res_data.insert(String::from("header"), Json::Object(header_data));

            let body:Json = res.body_json().await.unwrap();
            res_data.insert(String::from("body"), body);
            return Ok(Json::Object(res_data));
        },
        Err(e) => {
            return Err(Error::new("000", "response is not a json"));
        }
    }


    let mut res_data:HashMap<String, Value> = HashMap::new();




}