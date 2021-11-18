use async_std::path::{Path, PathBuf};
use chord::value::json;
use chord::value::map_merge_deep;
use chord::value::Value;
use dirs;
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone)]
pub struct Config {
    conf: Value,
}

impl Config {
    pub fn new(conf: Value) -> Config {
        let home_dir = dirs::home_dir()
            .map(|p| PathBuf::from(p).join(".chord"))
            .unwrap_or_else(|| Path::new("/").join("data").join("chord"));

        let conf_default = json!({
            "log": {
                "dir": home_dir.join("output").to_str().unwrap().to_string(),
                "level": {
                    "root": "warn",
                    "chord": "trace"
                }
            },

            "report": {
                "kind": "csv",
                "csv": {
                    "dir": home_dir.join("output").to_str().unwrap().to_string()
                }
            },

           "action": {
               "download": {
                   "workdir": home_dir.join("output").join("download").to_str().unwrap().to_string()
               },
               "dubbo": {
                   "enable": false,
                   "mode": "gateway",
                   "gateway": {
                       "lib": home_dir.join("lib").join("dubbo-generic-gateway-0.0.1-SNAPSHOT.jar").to_str().unwrap().to_string()
                   }
               },
               "docker": {
                   "enable": false
               },
               "dylib": {
                   "dir": home_dir.join("lib").to_str().unwrap().to_string()
               }
           }

        });

        let conf_merge = if conf.is_null() {
            conf_default.as_object().expect("invalid conf").clone()
        } else {
            map_merge_deep(
                conf_default.as_object().expect("invalid conf"),
                conf.as_object().expect("invalid conf"),
            )
        };

        Config {
            conf: Value::Object(conf_merge),
        }
    }
}

impl Config {
    pub fn log_dir(&self) -> &Path {
        let log_dir = self.conf["log"]["dir"].as_str().expect("invalid log.dir");
        Path::new(log_dir)
    }

    pub fn log_level(&self) -> Vec<(String, String)> {
        let target_level: Vec<(String, String)> = match self.conf["log"]["level"].as_object() {
            None => vec![],
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
        self.conf.get("report")
    }
}

impl Display for Config {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.write_str(format!("{}", self.conf).as_str())
    }
}
