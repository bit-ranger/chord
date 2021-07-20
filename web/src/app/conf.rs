use lazy_static::lazy_static;

use chord::value::json;
use chord::value::{Map, Value};

pub trait Config: Sync + Send {
    fn server_ip(&self) -> &str;

    fn server_port(&self) -> usize;

    fn log_path(&self) -> &str;

    fn job_input_path(&self) -> &str;

    fn ssh_key_private_path(&self) -> &str;

    fn log_level(&self) -> Vec<(String, String)>;

    fn report(&self) -> Option<&Value>;

    fn action(&self) -> Option<&Value>;
}

#[derive(Debug, Clone)]
pub struct ConfigImpl {
    conf: Value,
    report_default: Value,
}

impl ConfigImpl {
    pub fn new(conf: Value) -> ConfigImpl {
        let report_default = json!({ "csv": {
            "dir": "/data/chord/job/output"
        } });
        ConfigImpl {
            conf,
            report_default,
        }
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
            .unwrap_or("/data/chord/conf/id_rsa")
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

    fn report(&self) -> Option<&Value> {
        let report = self.conf.get("report");
        if report.is_some() {
            return report;
        }
        return Some(&self.report_default);
    }

    fn action(&self) -> Option<&Value> {
        self.conf.get("action")
    }
}
