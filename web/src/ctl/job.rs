use serde::{Serialize, Deserialize};
use chord_common::error::Error;
use validator::{Validate};
use std::time::SystemTime;
use chord_point::PointRunnerDefault;
use crate::biz;
use std::path::{PathBuf, Path};
use async_std::sync::Arc;
use chord_flow::AppContext;
use futures::executor::block_on;
use log::{error,debug,warn};
use git2::Repository;
use git2::build::RepoBuilder;
use lazy_static::lazy_static;
use regex::Regex;
use async_std::task::spawn_blocking;

lazy_static! {
    static ref GIT_URL: Regex = Regex::new(r"^git@[\w,.]+:[\w/-]+\.git$").unwrap();
}

#[test]
fn regex(){
    assert!(GIT_URL.is_match("git@github.com:bit-ranger/chord.git"));
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct Req {
    #[validate(regex = "GIT_URL")]
    git_url: String,

    #[validate(length(min = 1))]
    branch: Option<String>,

    #[validate(length(min = 1))]
    job_path: Option<String>
}

pub struct Ctl {
    input: PathBuf,
    output: PathBuf,
    ssh_key_private: PathBuf,
    app_ctx: Arc<dyn AppContext>,
    pool: rayon::ThreadPool
}

static mut JOB_CTL: Option<Ctl> = Option::None;

impl Ctl {
    pub fn new(input: &str,
               output: &str,
               ssh_key_private: &str,
    ) -> Ctl {
        Ctl {
            input: Path::new(input).to_path_buf(),
            output: Path::new(output).to_path_buf(),
            ssh_key_private: Path::new(ssh_key_private).to_path_buf(),
            app_ctx: block_on(chord_flow::create_app_context(Box::new(PointRunnerDefault::new()))),
            pool: rayon::ThreadPoolBuilder::new().build().unwrap(),
        }
    }

    pub async fn exec(&self, req: Req) -> Result<String, Error> {
        let exe_id = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_millis().to_string();
        let input = self.input.clone();
        let output = self.output.clone();
        let ssh_key_pri = self.ssh_key_private.clone();
        let app_ctx_0 = self.app_ctx.clone();
        let exe_id_0 = exe_id.clone();
        self.pool.spawn(|| block_on(Ctl::checkout_run(app_ctx_0, input, output, ssh_key_pri, req, exe_id_0)));
        return Ok(exe_id);
    }

    pub fn create_singleton(input: &str,
                            output: &str,
                            ssh_key_private: &str) -> &'static Ctl{
        unsafe {
            JOB_CTL = Some(Ctl::new(input, output, ssh_key_private));
            Ctl::get_singleton()
        }
    }

    pub fn get_singleton() -> &'static Ctl{
        unsafe {&JOB_CTL.as_ref().unwrap()}
    }

    async fn checkout_run(
        app_ctx: Arc<dyn AppContext>,
        input: PathBuf,
        output: PathBuf,
        ssh_key_pri: PathBuf,
        req: Req,
        exe_id: String) {
        let is_delimiter = |c: char| ['@',':','/'].contains(&c);
        let git_url_splits = Ctl::split(is_delimiter, req.git_url.as_str());

        let host = git_url_splits[1];
        let group_name = git_url_splits[2];
        let repo_name = git_url_splits[3];
        let checkout = input.clone()
            .join(host)
            .join(group_name)
            .join(repo_name);
        if checkout.exists() {
            if !checkout.is_dir(){
                error!("invalid checkout {}", checkout.to_str().unwrap());
                return;
            }
        } else {
            if let Err(e) = async_std::fs::create_dir_all(checkout.clone()).await {
                error!("create_dir checkout error {}, {}", checkout.to_str().unwrap(), e);
                return;
            }
        }

        if let Err(e) = Ctl::checkout(ssh_key_pri.as_path(),
                                      req.git_url.as_str(),
                                      checkout.as_path(),
                                      req.branch.unwrap_or_else(|| "master".to_owned()).as_str(),
        ).await {
            error!("checkout error {}, {}, {}", req.git_url, checkout.to_str().unwrap(), e);
            Ctl::clear(checkout.as_path()).await;
            return;
        }

        let job_path = match req.job_path{
            Some(p) => {
                let mut r = checkout.clone();
                for seg in p.split("/"){
                    r = r.join(seg);
                }
                r
            },
            None => checkout.clone()
        };

        let work_path = output
            .join(host)
            .join(group_name)
            .join(repo_name);
        Ctl::run(app_ctx, job_path, work_path, exe_id).await;
        Ctl::clear(checkout.as_path()).await;
    }

    async fn clear(dir: &Path) {
        let path = dir.to_owned();
        let result = spawn_blocking(move || {
            rm_rf::ensure_removed(path)
        }).await;

        match result{
            Ok(()) => debug!("remove dir {}", dir.to_str().unwrap()),
            Err(e) => warn!("remove dir {}, {}", dir.to_str().unwrap(), e),
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

    async fn checkout(ssh_key_private: &Path,
                      git_url: &str,
                      into: &Path,
                      branch: &str,
    ) -> Result<Repository, git2::Error> {
        let mut callbacks = git2::RemoteCallbacks::new();
        callbacks.credentials(|_, _, allowed| Ctl::credentials_callback(ssh_key_private, allowed));

        let mut fetch_opts = git2::FetchOptions::new();
        fetch_opts.remote_callbacks(callbacks);
        RepoBuilder::new()
            .branch(branch)
            .fetch_options(fetch_opts)
            .clone(git_url, into)
    }

    async fn run(app_ctx: Arc<dyn AppContext>,
                 job_path: PathBuf,
                 work_path: PathBuf,
                 exe_id: String) {
        let work_path = work_path.join(exe_id.as_str());

        if work_path.exists() {
            if !work_path.is_dir(){
                error!("invalid work_path {}", work_path.to_str().unwrap());
                return;
            }
        } else {
            if let Err(e) = async_std::fs::create_dir_all(work_path.clone()).await {
                error!("create_dir work_path error {}, {}", work_path.to_str().unwrap(), e);
                return;
            }
        }
        let _task_state_vec = biz::job::run(job_path, work_path, exe_id, app_ctx).await;
    }
}


