use std::time::SystemTime;

use async_std::sync::Arc;
use async_std::task::spawn;
use async_trait::async_trait;
use lazy_static::lazy_static;
use log::warn;
use regex::Regex;
use serde::{Deserialize, Serialize};
use validator::Validate;

use chord::value::Value;

use crate::app::conf::Config;
use chord::value::Map;
use chord_util::docker::container::Arg;
use chord_util::docker::engine::Engine;
use chord_util::docker::image::Image;

lazy_static! {
    static ref GIT_URL: Regex = Regex::new(r"^git@[\w,.]+:[\w/-]+\.git$").unwrap();
}

#[test]
fn regex() {
    assert!(GIT_URL.is_match("git@github.com:bit-ranger/chord.git"));
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("docker error:\n{0}")]
    Docker(chord_util::docker::Error),
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct Req {
    #[validate(regex = "GIT_URL")]
    git_url: String,

    #[validate(length(min = 1))]
    git_branch: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Rep {
    exec_id: String,
}

#[async_trait]
pub trait Ctl {
    async fn exec(&self, req: Req) -> Result<Rep, Error>;
}

pub struct CtlImpl {
    image: Arc<Image>,
    config: Arc<dyn Config>,
}

impl CtlImpl {
    pub async fn new(config: Arc<dyn Config>) -> Result<CtlImpl, Error> {
        let engine = Arc::new(
            Engine::new(config.docker_address().to_string())
                .await
                .map_err(|e| Error::Docker(e))?,
        );
        let image = Arc::new(
            Image::new(engine, config.docker_image())
                .await
                .map_err(|e| Error::Docker(e))?,
        );
        Ok(CtlImpl { image, config })
    }
}

#[async_trait]
impl Ctl for CtlImpl {
    async fn exec(&self, req: Req) -> Result<Rep, Error> {
        let req = Req {
            git_url: req.git_url,
            git_branch: Some(req.git_branch.unwrap_or("master".to_owned())),
        };

        let exec_id = (SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_millis()
            - 1622476800000)
            .to_string();
        let exec_id_0 = exec_id.clone();
        spawn(job_run(
            req,
            exec_id_0,
            self.config.clone(),
            self.image.clone(),
        ));
        return Ok(Rep { exec_id });
    }
}

async fn job_run(req: Req, exec_id: String, conf: Arc<dyn Config>, image: Arc<Image>) {
    let is_delimiter = |c: char| ['@', ':', '/'].contains(&c);
    let git_url_splits = split(is_delimiter, req.git_url.as_str());
    let host = git_url_splits[1];
    let group_name = git_url_splits[2];
    let repo_name = git_url_splits[3];
    let last_point_idx = repo_name.len() - 4;
    let repo_name = &repo_name.to_owned()[..last_point_idx];
    let job_name = format!("{}@{}@{}", repo_name, group_name, host).to_lowercase();

    let container_name = format!("chord-web-{}", exec_id);
    let ca = Arg::default();
    let env: Vec<String> = vec![
        format!("chord_exec_id={}", exec_id),
        format!("chord_job_name={}", job_name),
        format!("chord_git_url={}", req.git_url),
        format!("chord_git_branch={}", req.git_branch.unwrap()),
    ];

    let mut host_config = Map::new();
    host_config.insert(
        "Binds".to_string(),
        Value::Array(vec![Value::String(format!(
            "{}:/root",
            conf.workdir().to_str().unwrap()
        ))]),
    );

    let cmd = vec![
        "sh".to_string(),
        "/root/.chord/conf/chord-web-worker.sh".to_string(),
    ];

    let ca = ca.env(env).host_config(host_config).cmd(cmd);
    if let Err(e) = job_run_0(image, container_name, ca).await {
        warn!("job Err: {}, {}, {}", job_name, exec_id, e)
    }
}

async fn job_run_0(image: Arc<Image>, container_name: String, ca: Arg) -> Result<(), Error> {
    let mut container = image
        .container_create(container_name.as_str(), ca)
        .await
        .map_err(|e| Error::Docker(e))?;
    let _ = container.start().await.map_err(|e| Error::Docker(e))?;
    let wait_res = container.wait().await;
    match wait_res {
        Ok(_) => {
            let _ = container
                .tail(false, 100)
                .await
                .map_err(|e| Error::Docker(e))?;
        }
        Err(_) => {
            let _ = container
                .tail(true, 100)
                .await
                .map_err(|e| Error::Docker(e))?;
        }
    }
    Ok(())
}

fn split(is_delimiter: fn(char) -> bool, text: &str) -> Vec<&str> {
    let mut result: Vec<&str> = Vec::new();
    let mut li: usize = 0;
    for (i, c) in text.char_indices() {
        if is_delimiter(c) {
            result.push(&text[li..i]);
            li = i + 1;
        }
    }
    result.push(&text[li..]);
    return result;
}
