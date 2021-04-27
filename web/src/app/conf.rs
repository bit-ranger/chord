use chord_common::error::Error;
use chord_common::value::Json;

#[derive(Debug, Clone)]
pub struct App {
    conf: Json
}

impl App {

    pub fn new(conf: Json) -> Result<App,Error>{
        let app = App {
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
}
