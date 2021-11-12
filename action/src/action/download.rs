use std::str::FromStr;

use async_std::path::{Path, PathBuf};
use futures::io::BufWriter;
use log::{trace, warn};
use surf::http::headers::{HeaderName, HeaderValue};
use surf::http::Method;
use surf::{RequestBuilder, Response, Url};

use chord::action::prelude::*;

use crate::err;

pub struct DownloadFactory {
    workdir: PathBuf,
}

impl DownloadFactory {
    pub async fn new(config: Option<Value>) -> Result<DownloadFactory, Box<dyn Error>> {
        if config.is_none() {
            return Err(err!("100", "missing config"));
        }
        let config = config.as_ref().unwrap();

        if config.is_null() {
            return Err(err!("100", "missing config"));
        }

        let workdir = config["workdir"]
            .as_str()
            .ok_or(err!("101", "missing workdir"))?;

        let workdir = PathBuf::from_str(workdir)?;

        async_std::fs::create_dir_all(workdir.as_path()).await?;

        Ok(DownloadFactory { workdir })
    }
}

#[async_trait]
impl Factory for DownloadFactory {
    async fn create(&self, arg: &dyn CreateArg) -> Result<Box<dyn Action>, Box<dyn Error>> {
        let task_dir = self.workdir.join(arg.id().task_id().to_string());
        async_std::fs::create_dir_all(task_dir.as_path()).await?;
        trace!("tmp create {}", task_dir.as_path().to_str().unwrap());
        remove_dir(task_dir.as_path()).await;
        Ok(Box::new(Download { task_dir }))
    }
}

struct Download {
    task_dir: PathBuf,
}

#[async_trait]
impl Action for Download {
    async fn run(&self, arg: &dyn RunArg) -> Result<Box<dyn Scope>, Box<dyn Error>> {
        let file = run0(self, arg).await?;
        Ok(Box::new(file))
    }
}

async fn run0(
    download: &Download,
    arg: &dyn RunArg,
) -> std::result::Result<DownloadFile, Box<dyn Error>> {
    let args = arg.args()?;
    let url = args["url"].as_str().ok_or(err!("102", "missing url"))?;
    let url = Url::from_str(url).or(Err(err!("103", format!("invalid url: {}", url))))?;

    let mut rb = RequestBuilder::new(Method::Get, url);
    if let Some(header) = args["header"].as_object() {
        for (k, v) in header.iter() {
            let hn =
                HeaderName::from_string(k.clone()).or(Err(err!("104", "invalid header name")))?;
            let hvs: Vec<HeaderValue> = match v {
                Value::String(v) => {
                    vec![HeaderValue::from_str(v).or(Err(err!("105", "invalid header value")))?]
                }
                Value::Array(vs) => {
                    let mut vec = vec![];
                    for v in vs {
                        let v = HeaderValue::from_str(v.to_string().as_str())?;
                        vec.push(v)
                    }
                    vec
                }
                _ => Err(err!("106", "invalid header value"))?,
            };
            rb = rb.header(hn, hvs.as_slice());
        }
    }

    let case_dir = download.task_dir.join(arg.id().case_id().to_string());
    async_std::fs::create_dir_all(case_dir.as_path()).await?;
    let path = case_dir.join(arg.id().step());

    let mut df = DownloadFile { value: Value::Null };

    let file = async_std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open(path.as_path())
        .await?;
    let writer = BufWriter::new(file);

    let mut res: Response = rb.send().await?;
    let mut value = Map::new();
    value.insert(
        String::from("status"),
        Value::Number(Number::from_str(res.status().to_string().as_str()).unwrap()),
    );

    let mut header_data = Map::new();
    for (hn, hv) in res.iter() {
        header_data.insert(
            hn.to_string(),
            Value::Array(hv.iter().map(|v| Value::String(v.to_string())).collect()),
        );
    }
    value.insert(String::from("header"), Value::Object(header_data));

    let size = async_std::io::copy(res.take_body(), writer).await?;
    trace!("file create {}, {}", path.as_path().to_str().unwrap(), size);

    value.insert(String::from("size"), Value::Number(Number::from(size)));

    df.value = Value::Object(value);
    return Ok(df);
}

struct DownloadFile {
    value: Value,
}

impl Scope for DownloadFile {
    fn as_value(&self) -> &Value {
        &self.value
    }
}

async fn remove_dir(path: &Path) {
    let result = rm_rf::ensure_removed(std::path::Path::new(path));

    match result {
        Ok(()) => trace!("tmp remove {}", path.to_str().unwrap()),
        Err(e) => {
            if let rm_rf::Error::NotFound = e {
                trace!("tmp not found {}", path.to_str().unwrap());
            } else {
                warn!("tmp remove {}, {}", path.to_str().unwrap(), e);
            }
        }
    }
}
