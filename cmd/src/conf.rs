use chord::value::json::Json;

#[derive(Debug, Clone)]
pub struct Config {
    conf: Json,
}

impl Config {
    pub fn new(conf: Json) -> Config {
        Config { conf }
    }
}

impl Config {
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

    pub fn step_config(&self) -> Option<&Json> {
        self.conf.get("step")
    }
}
