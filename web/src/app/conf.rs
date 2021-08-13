use lazy_static::lazy_static;

use chord::value::{Map, Value};

pub trait Config: Sync + Send {
    fn server_ip(&self) -> &str;

    fn server_port(&self) -> usize;

    fn worker_shell_path(&self) -> &str;

    fn worker_key_path(&self) -> &str;

    fn docker_address(&self) -> &str;

    fn docker_image(&self) -> &str;

    fn cmd_conf_path(&self) -> &str;

    fn log_path(&self) -> &str;

    fn log_level(&self) -> Vec<(String, String)>;
}

#[derive(Debug, Clone)]
pub struct ConfigImpl {
    conf: Value,
}

impl ConfigImpl {
    pub fn new(conf: Value) -> ConfigImpl {
        ConfigImpl { conf }
    }
}

lazy_static! {
    static ref EMPTY_MAP: Map = Map::new();
}

impl Config for ConfigImpl {
    fn server_ip(&self) -> &str {
        self.conf["server"]["ip"].as_str().unwrap_or("127.0.0.1")
    }

    fn server_port(&self) -> usize {
        self.conf["server"]["port"].as_u64().unwrap_or(9999) as usize
    }

    fn worker_shell_path(&self) -> &str {
        self.conf["worker"]["shell"]["path"]
            .as_str()
            .unwrap_or("/data/chord/conf/chord_server_worker.sh")
    }

    fn worker_key_path(&self) -> &str {
        self.conf["worker"]["key"]["path"]
            .as_str()
            .unwrap_or("/data/chord/conf/id_rsa")
    }

    fn docker_address(&self) -> &str {
        self.conf["docker"]["address"]
            .as_str()
            .unwrap_or("127.0.0.1:2375")
    }

    fn docker_image(&self) -> &str {
        self.conf["docker"]["address"]
            .as_str()
            .unwrap_or("bitranger/chord:latest")
    }

    fn cmd_conf_path(&self) -> &str {
        self.conf["cmd"]["conf"]["path"]
            .as_str()
            .unwrap_or("/data/chord/conf/application.yml")
    }

    fn log_path(&self) -> &str {
        self.conf["log"]["path"]
            .as_str()
            .unwrap_or("/data/chord/job/output/web.log")
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
