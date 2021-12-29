use std::collections::VecDeque;
use std::str::FromStr;

use futures::StreamExt;
use log::trace;
use reqwest::header::{HeaderName, HeaderValue};
use reqwest::{Client, Method, Response, Url};

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
            Method::GET,
            None,
            999,
            |buf| String::from_utf8_lossy(buf).to_string(),
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
        self.call_with_op(uri, method, data, tail_size, |buf| {
            String::from_utf8_lossy(buf).to_string()
        })
        .await
    }

    pub async fn call_with_op<O: Fn(&Vec<u8>) -> String>(
        &self,
        uri: &str,
        method: Method,
        data: Option<Value>,
        tail_size: usize,
        op: O,
    ) -> Result<Vec<String>, Error> {
        call0(
            self.client.clone(),
            self.address.as_str(),
            uri,
            method,
            data,
            tail_size,
            op,
        )
        .await
        .map_err(|e| e.into())
    }
}

const CR: u8 = 0x0D;
const LF: u8 = 0x0A;

async fn call0<O: Fn(&Vec<u8>) -> String>(
    client: Client,
    address: &str,
    uri: &str,
    method: Method,
    data: Option<Value>,
    tail_size: usize,
    op: O,
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

    let res: Response = rb.send().await.map_err(|e| Error::Io(e.to_string()))?;

    let mut tail_lines: VecDeque<String> = VecDeque::with_capacity(tail_size);

    let status = res.status();

    let mut stream = res.bytes_stream();
    let mut line_buf = Vec::new();
    while let Some(bytes) = stream.next().await {
        let bytes = bytes.map_err(|e| Error::Io(e.to_string()))?;
        for b in bytes {
            if b == CR {
                // ignore
            } else if b == LF {
                end_of_line(&mut line_buf, &mut tail_lines, tail_size, &op);
            } else {
                line_buf.push(b);
            }
        }
    }
    end_of_line(&mut line_buf, &mut tail_lines, tail_size, &op);

    return if !status.is_success() {
        Err(Status(status.into()))
    } else {
        Ok(tail_lines.into())
    };
}

fn end_of_line<O: Fn(&Vec<u8>) -> String>(
    line_buf: &mut Vec<u8>,
    tail_lines: &mut VecDeque<String>,
    tail_size: usize,
    op: &O,
) {
    if line_buf.is_empty() {
        return;
    }
    let line: String = op(line_buf);
    line_buf.clear();

    trace!("{}", line);

    tail_lines.push_back(line.clone());
    if tail_lines.len() > tail_size {
        tail_lines.pop_front();
    }
}
