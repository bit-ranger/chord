use std::str::FromStr;

use async_std::prelude::*;
use log::trace;
use surf::http::headers::{HeaderName, HeaderValue};
use surf::http::Method;
use surf::{RequestBuilder, Response, Url};

use chord::value::Value;
use chord::Error;
use chord::{cause, err, rcause, rerr};

pub async fn call(
    address: &str,
    uri: &str,
    method: Method,
    data: Option<Value>,
    tail_size: usize,
) -> Result<Vec<String>, Error> {
    call0(address, uri, method, data, tail_size)
        .await
        .map_err(|e| e.into())
}

async fn call0(
    address: &str,
    uri: &str,
    method: Method,
    data: Option<Value>,
    tail_size: usize,
) -> Result<Vec<String>, Rae> {
    let url = format!("http://{}/{}", address, uri);
    let url = Url::from_str(url.as_str()).or(rerr!("docker", format!("invalid url: {}", url)))?;
    let mut rb = RequestBuilder::new(method, url);
    rb = rb.header(
        HeaderName::from_str("Content-Type").unwrap(),
        HeaderValue::from_str("application/json")?,
    );
    if let Some(d) = data {
        rb = rb.body(d);
    }

    let mut res: Response = rb.send().await?;

    let mut tail: Vec<String> = vec![];
    let mut line = String::new();
    loop {
        line.clear();
        let size = res
            .read_line(&mut line)
            .await
            .or_else(|e| rcause!("docker", "read fail", e))?;
        if size > 0 {
            if res.content_type().is_some()
                && res.content_type().unwrap().to_string() == "application/octet-stream"
            {
                line = String::from_utf8_lossy(&line.as_bytes()[8..]).to_string();
            }

            trace!("{}", line.trim_end());
            if tail.len() >= tail_size {
                tail.pop();
            }
            tail.push(line.clone());
        } else {
            break;
        }
    }
    return if !res.status().is_success() {
        rerr!("docker", res.status().to_string())?
    } else {
        Ok(tail)
    };
}

struct Rae(chord::Error);

impl From<surf::Error> for Rae {
    fn from(err: surf::Error) -> Rae {
        Rae(err!("docker", format!("{}", err.status())))
    }
}

impl From<chord::Error> for Rae {
    fn from(err: Error) -> Self {
        Rae(err)
    }
}

impl From<chord::value::Error> for Rae {
    fn from(err: chord::value::Error) -> Self {
        Rae(cause!("docker", "parse fail", err))
    }
}

impl Into<chord::Error> for Rae {
    fn into(self) -> Error {
        self.0
    }
}
