use async_std::path::{Path, PathBuf};
use chord::value::Value;

use dirs;

pub trait Config: Sync + Send {
    fn server_ip(&self) -> &str;

    fn server_port(&self) -> usize;

    fn docker_address(&self) -> &str;

    fn docker_image(&self) -> &str;

    fn workdir(&self) -> &Path;

    fn log_dir(&self) -> &Path;

    fn log_level(&self) -> Vec<(String, String)>;
}

#[derive(Debug, Clone)]
pub struct ConfigImpl {
    conf: Value,
    home_dir: PathBuf,
    log_dir: PathBuf,
    workdir: PathBuf,
}

impl ConfigImpl {
    pub fn new(conf: Value) -> ConfigImpl {
        let home_dir = dirs::home_dir()
            .map(|p| PathBuf::from(p).join(".chord"))
            .unwrap_or_else(|| Path::new("/").join("data").join("chord"));

        let workdir = match conf["workdir"].as_str() {
            Some(p) => Path::new(p).to_path_buf(),
            None => home_dir.join("web"),
        };

        let log_dir = match conf["log"]["dir"].as_str() {
            Some(p) => Path::new(p).to_path_buf(),
            None => home_dir.join("output"),
        };

        ConfigImpl {
            conf,
            home_dir,
            log_dir,
            workdir,
        }
    }
}

impl Config for ConfigImpl {
    fn server_ip(&self) -> &str {
        self.conf["server"]["ip"].as_str().unwrap_or("127.0.0.1")
    }

    fn server_port(&self) -> usize {
        self.conf["server"]["port"].as_u64().unwrap_or(9999) as usize
    }

    fn docker_address(&self) -> &str {
        self.conf["docker"]["address"]
            .as_str()
            .unwrap_or("127.0.0.1:2375")
    }

    fn docker_image(&self) -> &str {
        self.conf["docker"]["image"]
            .as_str()
            .unwrap_or("bitranger/chord:latest")
    }

    fn workdir(&self) -> &Path {
        self.log_dir.as_path()
    }

    fn log_dir(&self) -> &Path {
        self.log_dir.as_path()
    }

    fn log_level(&self) -> Vec<(String, String)> {
        let target_level: Vec<(String, String)> = match self.conf["log"]["level"].as_object() {
            None => Vec::new(),
            Some(m) => m
                .iter()
                .filter(|(_, v)| v.is_string())
                .map(|(k, v)| (k.to_owned(), v.as_str().unwrap().to_owned()))
                .collect(),
        };

        return target_level;
    }
}
