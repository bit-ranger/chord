use chord::err;
use chord::error::Error;
use chord::value::json::{Json, Map};
use lazy_static::lazy_static;

pub trait Config {
    fn server_ip(&self) -> &str;

    fn server_port(&self) -> usize;

    fn log_path(&self) -> &str;

    fn job_input_path(&self) -> &str;

    fn ssh_key_private_path(&self) -> &str;

    fn log_level(&self) -> Vec<(String, String)>;

    fn report_mongodb_url(&self) -> Result<&str, Error>;

    fn report_elasticsearch_url(&self) -> Result<&str, Error>;

    fn step_config(&self) -> Option<&Json>;
}

#[derive(Debug, Clone)]
pub struct ConfigImpl {
    conf: Json,
}

impl ConfigImpl {
    pub fn new(conf: Json) -> ConfigImpl {
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

    fn log_path(&self) -> &str {
        self.conf["log"]["path"]
            .as_str()
            .unwrap_or("/data/chord/job/output/web.log")
    }

    fn job_input_path(&self) -> &str {
        self.conf["job"]["input"]["path"]
            .as_str()
            .unwrap_or("/data/chord/job/input")
    }

    fn ssh_key_private_path(&self) -> &str {
        self.conf["ssh"]["key"]["private"]["path"]
            .as_str()
            .unwrap_or("/data/chord/conf/ssh_key.pri")
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

    fn report_mongodb_url(&self) -> Result<&str, Error> {
        self.conf["report"]["mongodb"]["url"]
            .as_str()
            .ok_or(err!("config", "missing report.mongodb.url"))
    }

    fn report_elasticsearch_url(&self) -> Result<&str, Error> {
        self.conf["report"]["elasticsearch"]["url"]
            .as_str()
            .ok_or(err!("config", "missing report.mongodb.url"))
    }

    fn step_config(&self) -> Option<&Json> {
        self.conf.get("step")
    }
}
