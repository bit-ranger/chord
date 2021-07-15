use chord::value::Value;

#[derive(Debug, Clone)]
pub struct Config {
    conf: Value,
}

impl Config {
    pub fn new(conf: Value) -> Config {
        Config { conf }
    }
}

impl Config {
    pub fn log_path(&self) -> &str {
        self.conf["log"]["path"]
            .as_str()
            .unwrap_or("/data/chord/job/output/cmd.log")
    }

    pub fn log_level(&self) -> Vec<(String, String)> {
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

    pub fn action(&self) -> Option<&Value> {
        self.conf.get("action")
    }

    pub fn report(&self) -> Option<&Value> {
        self.conf.get("report")
    }
}
