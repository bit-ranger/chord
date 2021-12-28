use std::collections::VecDeque;
use std::str::FromStr;

use futures::AsyncBufReadExt;
use log::trace;
use reqwest::header::{HeaderName, HeaderValue};
use reqwest::{Body, Client, Method, Response, Url};

use chord_core::value::Value;

use crate::docker::Error;
use crate::docker::Error::*;

pub struct Engine {
    address: String,
    client: Client,
}

impl Engine {
    pub async fn new(address: String) -> Result<Engine, Error> {
        trace!("docker info {}", address);
        let client = Client::new();
        call0(
            client.clone(),
            address.as_str(),
            "info",
            Method::Get,
            None,
            999,
        )
        .await
        .map_err(|e| e.into())
        .map(|_| Engine { address, client })
    }

    pub async fn call(
        &self,
        uri: &str,
        method: Method,
        data: Option<Value>,
        tail_size: usize,
    ) -> Result<Vec<String>, Error> {
        call0(
            self.client.clone(),
            self.address.as_str(),
            uri,
            method,
            data,
            tail_size,
        )
        .await
        .map_err(|e| e.into())
    }
}

async fn call0(
    client: Client,
    address: &str,
    uri: &str,
    method: Method,
    data: Option<Value>,
    tail_size: usize,
) -> Result<Vec<String>, Error> {
    let url = format!("http://{}/{}", address, uri);
    let url = Url::from_str(url.as_str()).or(Err(Error::Url(url)))?;
    let mut rb = client.request(method, url);
    rb = rb.header(
        HeaderName::from_str("Content-Type").unwrap(),
        HeaderValue::from_str("application/json").unwrap(),
    );
    if let Some(d) = data {
        rb = rb.body(d.to_string());
    }

    let mut res: Response = rb.send().await.map_err(|e| Error::Io(e.to_string()))?;

    let mut tail_lines: VecDeque<String> = VecDeque::with_capacity(tail_size);
    let mut line_buf = vec![];
    loop {
        line_buf.clear();
        let size = res
            .bytes()
            .await
            .map_err(|e| Error::Io(e.to_string()))?
            .reader()
            .read_until(b'\n', &mut line_buf)
            .await
            .or_else(|e| Err(Io(format!("{}", e))))?;
        if size > 0 {
            let line = if res.content_type().is_some()
                && res.content_type().unwrap().to_string() == "application/octet-stream"
            {
                String::from_utf8_lossy(&line_buf[8..]).to_string()
            } else {
                String::from_utf8_lossy(&line_buf).to_string()
            };
            let line = format!("{}\n", line);

            trace!("{}", line);

            tail_lines.push_back(line.clone());
            if tail_lines.len() > tail_size {
                tail_lines.pop_front();
            }
        } else {
            break;
        }
    }
    return if !res.status().is_success() {
        Err(Status(res.status().into()))
    } else {
        Ok(tail_lines.into())
    };
}
