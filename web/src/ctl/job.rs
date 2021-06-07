use std::path::{Path, PathBuf};
use std::time::SystemTime;

use async_std::sync::Arc;
use async_std::task::{spawn, spawn_blocking};
use git2::build::RepoBuilder;
use git2::Repository;
use lazy_static::lazy_static;
use log::{error, warn, trace};
use regex::Regex;
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::app::conf::Config;
pub use async_trait::async_trait;
use chord_common::error::Error;
use chord_flow::Context;
use chord_step::StepRunnerFactoryDefault;

use crate::biz;

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
    branch: Option<String>,

    #[validate(length(min = 1))]
    job_path: Option<String>,
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
    input_dir: PathBuf,
    ssh_key_private: PathBuf,
    flow_ctx: Arc<dyn Context>,
    config: Arc<dyn Config>,
}

unsafe impl Send for CtlImpl {}

unsafe impl Sync for CtlImpl {}

impl CtlImpl {
    pub async fn new(config: Arc<dyn Config>) -> Result<CtlImpl, Error> {
        Ok(CtlImpl {
            input_dir: Path::new(config.job_input_path()).to_path_buf(),
            ssh_key_private: Path::new(config.ssh_key_private_path()).to_path_buf(),
            flow_ctx: chord_flow::context_create(Box::new(
                StepRunnerFactoryDefault::new(config.step_config().clone()).await?,
            ))
            .await,
            config,
        })
    }
}

#[async_trait]
impl Ctl for CtlImpl {
    async fn exec(&self, req: Req) -> Result<Rep, Error> {
        let req = Req {
            git_url: req.git_url,
            branch: Some(req.branch.unwrap_or("master".to_owned())),
            job_path: Some(req.job_path.unwrap_or("/".to_owned()))
        };

        let exec_id = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_millis()
            .to_string();
        let input = self.input_dir.clone();
        let ssh_key_pri = self.ssh_key_private.clone();
        let app_ctx_0 = self.flow_ctx.clone();
        let exec_id_0 = exec_id.clone();
        spawn(checkout_run(
            app_ctx_0,
            input,
            ssh_key_pri,
            req,
            exec_id_0,
            self.config.report_elasticsearch_url()?.to_owned(),
        ));
        return Ok(Rep { exec_id });
    }
}

async fn checkout_run(
    app_ctx: Arc<dyn Context>,
    input: PathBuf,
    ssh_key_pri: PathBuf,
    req: Req,
    exec_id: String,
    es_url: String,
) {
    trace!("checkout_run {:?}", req);

    let is_delimiter = |c: char| ['@', ':', '/'].contains(&c);
    let git_url_splits = split(is_delimiter, req.git_url.as_str());

    let host = git_url_splits[1];
    let group_name = git_url_splits[2];
    let repo_name = git_url_splits[3];
    let last_step_idx = repo_name.len() - 4;
    let repo_name = &repo_name.to_owned()[..last_step_idx];
    let checkout_path = input.clone().join(host).join(group_name).join(repo_name);

    if checkout_path.exists() {
        error!("checkout exist {}", checkout_path.to_str().unwrap());
        return;
    } else {
        if let Err(e) = async_std::fs::create_dir_all(checkout_path.clone()).await {
            error!(
                "checkout create_dir error {}, {}",
                checkout_path.to_str().unwrap(),
                e
            );
            return;
        }
    }

    if let Err(e) = checkout(
        ssh_key_pri.as_path(),
        req.git_url.as_str(),
        checkout_path.as_path(),
        req.branch.as_ref().unwrap().as_str(),
    )
    .await
    {
        error!(
            "checkout error {}, {}, {}",
            req.git_url,
            checkout_path.to_str().unwrap(),
            e
        );
        clear(checkout_path.as_path()).await;
        return;
    }

    let job_path = match req.job_path {
        Some(p) => {
            let mut r = checkout_path.clone();
            for seg in p.split("/") {
                r = r.join(seg);
            }
            r
        }
        None => checkout_path.clone(),
    };

    let job_name = format!("{}@{}@{}", repo_name, group_name, host).to_lowercase();
    job_run(app_ctx, job_path, job_name, exec_id, es_url).await;
    clear(checkout_path.as_path()).await;
}

async fn clear(dir: &Path) {
    let path = dir.to_owned();
    let result = spawn_blocking(move || rm_rf::ensure_removed(path)).await;

    match result {
        Ok(()) => trace!("remove dir {}", dir.to_str().unwrap()),
        Err(e) => error!("remove dir {}, {}", dir.to_str().unwrap(), e),
    }
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

fn credentials_callback<P: AsRef<Path>>(
    ssh_key_private: P,
    cred_types_allowed: git2::CredentialType,
) -> Result<git2::Cred, git2::Error> {
    if cred_types_allowed.contains(git2::CredentialType::SSH_KEY) {
        let cred = git2::Cred::ssh_key("git", None, ssh_key_private.as_ref(), None);
        if let Err(e) = &cred {
            error!("ssh_key error {}", e);
        }
        return cred;
    }

    return Err(git2::Error::from_str("ssh_key not allowed"));
}

async fn checkout(
    ssh_key_private: &Path,
    git_url: &str,
    into: &Path,
    branch: &str,
) -> Result<Repository, git2::Error> {
    let mut callbacks = git2::RemoteCallbacks::new();
    callbacks.credentials(|_, _, allowed| credentials_callback(ssh_key_private, allowed));

    let mut fetch_opts = git2::FetchOptions::new();
    fetch_opts.remote_callbacks(callbacks);
    RepoBuilder::new()
        .branch(branch)
        .fetch_options(fetch_opts)
        .clone(git_url, into)
}

async fn job_run(
    app_ctx: Arc<dyn Context>,
    job_path: PathBuf,
    job_name: String,
    exec_id: String,
    es_url: String,
) {
    let job_result = biz::job::run(job_path, job_name, exec_id, app_ctx, es_url).await;
    if let Err(e) = job_result {
        warn!(
            "job run error {}",
            e
        );
    }
}