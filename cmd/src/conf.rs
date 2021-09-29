use async_std::path::{Path, PathBuf};
use chord::value::json;
use chord::value::Value;
use dirs;

#[derive(Debug, Clone)]
pub struct Config {
    conf: Value,
    home_dir: PathBuf,
    log_dir: PathBuf,
    report_default: Value,
}

impl Config {
    pub fn new(conf: Value) -> Config {
        let home_dir = dirs::home_dir()
            .map(|p| PathBuf::from(p).join(".chord"))
            .unwrap_or_else(|| Path::new("/").join("data").join("chord"));

        let report_default = json!({ "csv": {
            "dir": home_dir.join("output").to_str().unwrap().to_string()
        } });

        let log_dir = match conf["log"]["dir"].as_str() {
            Some(p) => Path::new(p).to_path_buf(),
            None => home_dir.join("output"),
        };

        Config {
            conf,
            home_dir: home_dir.clone(),
            log_dir,
            report_default,
        }
    }
}

impl Config {
    pub fn log_dir(&self) -> &Path {
        self.log_dir.as_path()
    }

    pub fn log_level(&self) -> Vec<(String, String)> {
        let target_level: Vec<(String, String)> = match self.conf["log"]["level"].as_object() {
            None => vec![
                ("root".to_string(), "warn".to_string()),
                ("chord".to_string(), "trace".to_string()),
            ],
            Some(m) => m
                .iter()
                .filter(|(_, v)| v.is_string())
                .map(|(k, v)| (k.to_owned(), v.as_str().unwrap().to_owned()))
                .collect(),
        };

        return target_level;
    }

    pub fn action(&self) -> Option<&Value> {
        self.conf.get("action")
    }

    pub fn report(&self) -> Option<&Value> {
        let report = self.conf.get("report");
        if report.is_some() {
            return report;
        }
        return Some(&self.report_default);
    }
}
