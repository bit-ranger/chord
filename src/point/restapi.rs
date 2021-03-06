use crate::model::{Error, Json, PointResult, PointContext};
use surf::{RequestBuilder, Response, Url};
use surf::http::Method;
use std::str::FromStr;
use surf::http::headers::{HeaderName, HeaderValue};
use serde_json::Map;

pub async fn run(context: &dyn PointContext) -> PointResult{
    let url = match context.get_config_rendered(vec!["url"]){
        Some(url) => url,
        None => return Err(Error::new("010", "missing url"))
    };

    let url =match Url::from_str(url.as_str()){
        Ok(url) => url,
        Err(_) => return Err(Error::new("011", "invalid url"))
    };

    let method = match context.get_config_rendered(vec!["method"]){
        Some(method) => method,
        None => return Err(Error::new("020", "missing method"))
    };

    let method = match Method::from_str(method.as_str()){
        Ok(method) => method,
        Err(_) => return Err(Error::new("021", "invalid method"))
    };

    let mut rb = RequestBuilder::new(method, url);

    if let Some(header) = context.get_config()["header"].as_object() {
        for (k,v) in header.iter() {
            let hn = HeaderName::from_str(context.render(k).as_str());
            if hn.is_err(){
                return Err(Error::new("030","invalid header name"));
            }
            let hv= HeaderValue::from_str(context.render(v.as_str().unwrap()).as_str());
            if hv.is_err(){
                return Err(Error::new("031","invalid header value"));
            }
            rb = rb.header(hn.unwrap(), hv.unwrap());
        }
    }

    let  response: surf::Result<Response> = rb.send().await;
    return match response {
        Ok(mut res) => {
            let mut res_data = Map::<String, Json>::new();
            res_data.insert(String::from("status"), Json::String(String::from(res.status().to_string())));

            let mut header_data: Map<String, Json> = Map::new();
            for header_name in res.header_names() {
                header_data.insert(header_name.to_string(), Json::String(res.header(header_name).unwrap().to_string()));
            }

            res_data.insert(String::from("header"), Json::Object(header_data));

            let body: Json = res.body_json().await.unwrap();
            res_data.insert(String::from("body"), body);
            Ok(Json::Object(res_data))
        },
        Err(e) => {
            Err(Error::new("000", format!("http error: {}", e).as_str()))
        }
    }

}