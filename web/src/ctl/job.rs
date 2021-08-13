use std::time::SystemTime;

use async_std::sync::Arc;
use async_std::task::spawn;
use async_trait::async_trait;
use lazy_static::lazy_static;
use log::warn;
use regex::Regex;
use serde::{Deserialize, Serialize};
use validator::Validate;

use chord::value::json;
use chord::Error;

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
        let engine = Arc::new(Engine::new(config.docker_address().to_string()).await?);
        let image = Arc::new(Image::new(engine, config.docker_image()).await?);
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

    let mut volumes = Map::new();
    volumes.insert(
        conf.worker_key_path().to_string(),
        json!({
                "Target": "/data/chord/conf/id_rsa" ,
                "Source": "volume1" ,
                "Type": "volume",
                "ReadOnly": false
        }),
    );
    volumes.insert(
        conf.worker_shell_path().to_string(),
        json!({
                "Target": "/usr/bin/chord-web-worker.sh" ,
                "Source": "volume2" ,
                "Type": "volume",
                "ReadOnly": false
        }),
    );
    volumes.insert(
        conf.cmd_conf_path().to_string(),
        json!({
                "Target": "/data/chord/conf/cmd.yml" ,
                "Source": "volume3" ,
                "Type": "volume",
                "ReadOnly": false
        }),
    );
    volumes.insert(
        "/data/chord/job/output".to_string(),
        json!({
                "Target": "/data/chord/job/output" ,
                "Source": "volume4" ,
                "Type": "volume",
                "ReadOnly": false
        }),
    );

    let cmd = vec!["chord_server_worker.sh".to_string()];

    let ca = ca.env(env).volumes(volumes).cmd(cmd);
    if let Err(e) = job_run_0(image, container_name, ca).await {
        warn!("job Err: {}, {}, {}", job_name, exec_id, e)
    }
}

async fn job_run_0(image: Arc<Image>, container_name: String, ca: Arg) -> Result<(), Error> {
    let mut container = image.container_create(container_name.as_str(), ca).await?;
    let _ = container.start().await?;
    let _ = container.wait().await?;
    let _ = container.tail(100).await?;
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
