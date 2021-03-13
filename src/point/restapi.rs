use std::str::FromStr;

use surf::{RequestBuilder, Response, Url, Body};
use surf::http::headers::{HeaderName, HeaderValue};
use surf::http::Method;

use crate::model::context::{PointContext, PointResult};
use crate::model::error::Error;
use crate::model::value::{Json, Map, Number};

pub async fn run(context: &dyn PointContext) -> PointResult {
    let url = match context.get_config_rendered(vec!["url"]) {
        Some(url) => url,
        None => return Err(Error::new("010", "missing url"))
    };

    let url = match Url::from_str(url.as_str()) {
        Ok(url) => url,
        Err(_) => return Err(Error::new("011", format!("invalid url: {}", url).as_str()))
    };

    let method = match context.get_config_rendered(vec!["method"]) {
        Some(method) => method,
        None => return Err(Error::new("020", "missing method"))
    };

    let method = match Method::from_str(method.as_str()) {
        Ok(method) => method,
        Err(_) => return Err(Error::new("021", "invalid method"))
    };



    let mut rb = RequestBuilder::new(method, url);

    if let Some(header) = context.get_config()["header"].as_object() {
        for (k, v) in header.iter() {
            let hn = HeaderName::from_str(context.render(k)?.as_str());
            if hn.is_err() {
                return Err(Error::new("030", "invalid header name"));
            }
            let hv = HeaderValue::from_str(context.render(v.as_str().unwrap())?.as_str());
            if hv.is_err() {
                return Err(Error::new("031", "invalid header value"));
            }
            rb = rb.header(hn.unwrap(), hv.unwrap());
        }
    }

    if let Some(body) = context.get_config_rendered(vec!["body"]){
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

    // return match response {
    //     Ok(mut res) => {
    //         let mut res_data = Map::new();
    //         res_data.insert(String::from("status"), Json::Number(Number::from_str(res.status().to_string().as_str()).unwrap()));
    //
    //         let mut header_data = Map::new();
    //         for header_name in res.header_names() {
    //             header_data.insert(header_name.to_string(), Json::String(res.header(header_name).unwrap().to_string()));
    //         }
    //
    //         res_data.insert(String::from("header"), Json::Object(header_data));
    //
    //         let body: Json = res.body_json().await?;
    //         res_data.insert(String::from("body"), body);
    //         Ok(Json::Object(res_data))
    //     }
    //     Err(e) => {
    //         Err(Error::new("000", format!("http error: {}", e).as_str()))
    //     }
    // };

}

// impl std::error::Error for surf::Error {
//     fn description(&self) -> &str {
//         // format!("{} \"code\": \"{}\", \"message\": \"{}\" {}", "{", self.code, self.message, "}").as_str()
//         "surf error"
//     }
//
//     fn cause(self: &surf::Error) -> Option<&dyn std::error::Error> {
//         None
//     }
// }

impl From<surf::Error> for Error {
    fn from(err: surf::Error) -> Error {
        Error::new("http", format!("{}", err.status()).as_str())
    }
}


