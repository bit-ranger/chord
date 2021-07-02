use std::str::FromStr;

use async_std::io::BufWriter;
use async_std::path::PathBuf;
use futures::executor::block_on;
use futures::AsyncWriteExt;
use log::{trace, warn};

use chord::action::prelude::*;

pub struct DownloadFactory {
    workdir: PathBuf,
}

impl DownloadFactory {
    pub async fn new(config: Option<Value>) -> Result<DownloadFactory, Error> {
        if config.is_none() {
            return rerr!("010", "missing config");
        }
        let config = config.as_ref().unwrap();

        if config.is_null() {
            return rerr!("010", "missing config");
        }

        let workdir = config["workdir"]
            .as_str()
            .ok_or(err!("010", "missing workdir"))?;

        let workdir = PathBuf::from_str(workdir)?;

        async_std::fs::create_dir_all(workdir.as_path()).await?;

        Ok(DownloadFactory { workdir })
    }
}

#[async_trait]
impl Factory for DownloadFactory {
    async fn create(&self, arg: &dyn CreateArg) -> Result<Box<dyn Action>, Error> {
        let tmp = self.workdir.join(arg.id());
        async_std::fs::create_dir_all(tmp.as_path()).await?;
        trace!("tmp create {}", tmp.as_path().to_str().unwrap());
        Ok(Box::new(Download {
            name: arg.id().into(),
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
        let url = arg.render_str(
            arg.args()["url"]
                .as_str()
                .ok_or(err!("010", "missing url"))?,
        )?;
        let path = self.tmp.join(arg.id());
        let file = async_std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open(path.as_path())
            .await?;
        let mut writer = BufWriter::new(file);
        writer.write_all(url.as_bytes()).await?;
        trace!("file create {}", path.as_path().to_str().unwrap());

        let download_file = DownloadFile {
            value: Value::Array(vec![
                Value::String(self.name.clone()),
                Value::String(arg.id().into()),
            ]),
            path,
        };
        return Ok(Box::new(download_file));
    }
}

impl Drop for Download {
    fn drop(&mut self) {
        let path = self.tmp.clone();
        let result = rm_rf::ensure_removed(path);

        match result {
            Ok(()) => trace!("tmp remove {}", self.tmp.as_path().to_str().unwrap()),
            Err(e) => warn!("tmp remove {}, {}", self.tmp.as_path().to_str().unwrap(), e),
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
            Err(e) => warn!(
                "file remove {}, {}",
                self.path.as_path().to_str().unwrap(),
                e
            ),
        }
    }
}
