use std::str::FromStr;

use async_std::path::PathBuf;
use log::trace;

use chord::action::prelude::*;
use chord::value::{Map, Number};

pub struct FstoreFactory {
    workdir: PathBuf,
}

impl FstoreFactory {
    pub async fn new(config: Option<Value>) -> Result<FstoreFactory, Error> {
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

        Ok(FstoreFactory { workdir })
    }
}

#[async_trait]
impl Factory for FstoreFactory {
    async fn create(&self, arg: &dyn CreateArg) -> Result<Box<dyn Action>, Error> {
        let tmp = self.workdir.join(arg.id().to_string());
        async_std::fs::create_dir_all(tmp.as_path()).await?;
        trace!("tmp create {}", tmp.as_path().to_str().unwrap());
        Ok(Box::new(Fstore {
            name: arg.id().to_string(),
            tmp,
        }))
    }
}

struct Fstore {
    name: String,
    tmp: PathBuf,
}

#[async_trait]
impl Action for Fstore {
    async fn run(&self, arg: &dyn RunArg) -> Result<Box<dyn Scope>, Error> {
        let file = run0(self, arg).await?;
        Ok(Box::new(file))
    }
}

async fn run0(fstore: &Fstore, arg: &dyn RunArg) -> std::result::Result<Value, Error> {
    let args = arg.args(None)?;
    let pav: Vec<String> = args["path"]
        .as_array()
        .ok_or(err!("102", "missing path"))?
        .iter()
        .map(|p| p.as_str())
        .filter(|p| p.is_some())
        .map(|p| p.unwrap())
        .map(|p| p.to_owned())
        .collect();

    if pav.len() < 1 {
        return Err(err!("103", "missing path"));
    }
    if !pav[0].starts_with(arg.id().case_id().task_id().to_string().as_str()) {
        return Err(err!("104", "forbidden access"));
    }

    let mut path_src = fstore.tmp.parent().unwrap().to_path_buf();
    for pa in pav {
        path_src = path_src.join(pa.as_str());
    }
    let path_dest = fstore.tmp.join(arg.id().to_string());

    let size = async_std::fs::copy(path_src, path_dest).await?;

    let mut value = Map::new();
    value.insert(
        String::from("path"),
        Value::Array(vec![
            Value::String(fstore.name.clone()),
            Value::String(arg.id().to_string()),
        ]),
    );
    value.insert(String::from("size"), Value::Number(Number::from(size)));
    return Ok(Value::Object(value));
}
