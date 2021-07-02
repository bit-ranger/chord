use std::str::FromStr;

use async_std::io::BufWriter;
use async_std::path::PathBuf;
use futures::executor::block_on;
use futures::AsyncWriteExt;
use log::{debug, error, info};

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
        debug!("tmp create {}", tmp.as_path().to_str().unwrap());
        Ok(Box::new(Download { tmp }))
    }
}

struct Download {
    tmp: PathBuf,
}

#[async_trait]
impl Action for Download {
    async fn run(&self, arg: &dyn RunArg) -> Result<Box<dyn Scope>, Error> {
        // let url = arg.render_str(arg.args()["url"].as_str()?)?;
        let file_path = self.tmp.join(arg.id());
        let file = async_std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open(file_path.as_path())
            .await?;
        let mut writer = BufWriter::new(file);
        writer.write_all(format!("hello world!").as_bytes());
        debug!("download {}", file_path.as_path().to_str().unwrap());
        return Ok(Box::new(config));
    }
}

impl Drop for Download {
    fn drop(&mut self) {
        let path = self.tmp.clone();
        let result = rm_rf::ensure_removed(path);

        match result {
            Ok(()) => debug!("tmp remove {}", self.tmp.as_path().to_str().unwrap),
            Err(e) => error!("tmp remove {}, {}", self.tmp.as_path().to_str().unwrap, e),
        }
    }
}

struct DownloadFile {
    path: PathBuf,
    name: Value,
}

impl Scope for DownloadFile {
    fn as_value(&self) -> &Value {
        &self.name
    }
}

impl Drop for DownloadFile {
    fn drop(&mut self) {
        let result = block_on(async || async_std::fs::remove_file(self.path.as_path()).await);

        match result {
            Ok(()) => debug!("file remove {}", self.tmp.as_path().to_str().unwrap),
            Err(e) => error!("file remove {}, {}", self.tmp.as_path().to_str().unwrap, e),
        }
    }
}
