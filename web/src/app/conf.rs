use chord_common::error::Error;
use chord_common::value::Json;

#[derive(Debug, Clone)]
pub struct Config {
    conf: Json
}

impl Config {

    pub fn new(conf: Json) -> Result<Config,Error>{
        let app = Config {
            conf
        };
        return Ok(app);
    }

    pub fn server_ip(&self) -> &str {
        self.conf["server"]["ip"].as_str().unwrap_or("127.0.0.1")
    }

    pub fn server_port(&self) -> usize {
        self.conf["server"]["port"].as_u64().unwrap_or(9999) as usize
    }

    pub fn log_path(&self) -> &str {
        self.conf["log"]["path"].as_str().unwrap_or("/data/chord/job/output/web.log")
    }

    pub fn job_input_path(&self) -> &str {
        self.conf["job"]["input"]["path"].as_str().unwrap_or("/data/chord/job/input")
    }

    pub fn job_output_path(&self) -> &str {
        self.conf["job"]["output"]["path"].as_str().unwrap_or("/data/chord/job/output")
    }

    pub fn ssh_key_private_path(&self) -> &str {
        self.conf["ssh"]["key"]["private"]["path"].as_str().unwrap_or("/data/chord/conf/ssh_key.pri")
    }

    pub fn log_level(&self) -> Vec<(String, String)>{
        let target_level: Vec<(String, String)> =  match self.conf["log"]["level"]
            .as_object(){
            None => Vec::new(),
            Some(m) => {
                m.iter()
                    .filter(|(_,v)| v.is_string())
                    .map(|(k,v)| (k.to_owned(), v.as_str().unwrap().to_owned()))
                    .collect()
            }
        };

        return target_level;
    }
}
