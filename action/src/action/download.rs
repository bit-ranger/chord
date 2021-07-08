use std::str::FromStr;

use async_std::io::BufWriter;
use async_std::path::PathBuf;
use futures::executor::block_on;
use log::{trace, warn};
use surf::http::headers::{HeaderName, HeaderValue};
use surf::http::Method;
use surf::{RequestBuilder, Response, Url};

use chord::action::prelude::*;
use chord::value::{Map, Number};

pub struct DownloadFactory {
    workdir: PathBuf,
}

impl DownloadFactory {
    pub async fn new(config: Option<Value>) -> Result<DownloadFactory, Error> {
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
    async fn create(&self, arg: &dyn CreateArg) -> Result<Box<dyn Action>, Error> {
        let tmp = self.workdir.join(arg.id().to_string());
        async_std::fs::create_dir_all(tmp.as_path()).await?;
        trace!("tmp create {}", tmp.as_path().to_str().unwrap());
        Ok(Box::new(Download {
            name: arg.id().to_string(),
            tmp,
        }))
    }
}

struct Download {
    name: String,
    tmp: PathBuf,
}

#[async_trait]
impl Action for Download {
    async fn run(&self, arg: &dyn RunArg) -> Result<Box<dyn Scope>, Error> {
        let file = run0(self, arg).await.map_err(|e| e.0)?;
        Ok(Box::new(file))
    }
}

async fn run0(
    download: &Download,
    arg: &dyn RunArg,
) -> std::result::Result<DownloadFile, DownloadError> {
    let args = arg.render_value(arg.args())?;
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

    let path = download.tmp.join(arg.id().to_string());

    let mut df = DownloadFile {
        path: path.clone(),
        value: Value::Null,
    };

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

    value.insert(
        String::from("path"),
        Value::Array(vec![
            Value::String(download.name.clone()),
            Value::String(arg.id().to_string()),
        ]),
    );

    let size = async_std::io::copy(res.take_body(), writer).await?;
    trace!("file create {}, {}", path.as_path().to_str().unwrap(), size);

    value.insert(String::from("size"), Value::Number(Number::from(size)));

    df.value = Value::Object(value);
    return Ok(df);
}

impl Drop for Download {
    fn drop(&mut self) {
        let path = self.tmp.clone();
        let result = rm_rf::ensure_removed(path);

        match result {
            Ok(()) => trace!("tmp remove {}", self.tmp.as_path().to_str().unwrap()),
            Err(e) => {
                if let rm_rf::Error::NotFound = e {
                    trace!("tmp not found {}", self.tmp.as_path().to_str().unwrap());
                } else {
                    warn!("tmp remove {}, {}", self.tmp.as_path().to_str().unwrap(), e);
                }
            }
        }
    }
}

struct DownloadFile {
    path: PathBuf,
    value: Value,
}

impl Scope for DownloadFile {
    fn as_value(&self) -> &Value {
        &self.value
    }
}

impl Drop for DownloadFile {
    fn drop(&mut self) {
        let result = block_on(async_std::fs::remove_file(self.path.as_path()));

        match result {
            Ok(()) => trace!("file remove {}", self.path.as_path().to_str().unwrap()),
            Err(e) => {
                if let std::io::ErrorKind::NotFound = e.kind() {
                    trace!("file not found {}", self.path.as_path().to_str().unwrap());
                } else {
                    warn!(
                        "file remove {}, {}",
                        self.path.as_path().to_str().unwrap(),
                        e
                    );
                }
            }
        }
    }
}

struct DownloadError(chord::Error);

impl From<surf::Error> for DownloadError {
    fn from(err: surf::Error) -> DownloadError {
        DownloadError(err!("107", format!("{}", err.status())))
    }
}

impl From<chord::Error> for DownloadError {
    fn from(err: Error) -> Self {
        DownloadError(err)
    }
}

impl From<std::io::Error> for DownloadError {
    fn from(err: std::io::Error) -> Self {
        DownloadError(cause!("108", err.to_string(), err))
    }
}
