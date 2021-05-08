use chord_common::error::Error;
use chord_common::value::Json;
use chord_common::err;
use shaku::Component;
use shaku::Interface;


pub trait Config: Interface{

    fn server_ip(&self) -> &str; 

    fn server_port(&self) -> usize ;

    fn log_path(&self) -> &str ;

    fn job_input_path(&self) -> &str ; 

    fn ssh_key_private_path(&self) -> &str ;

    fn log_level(&self) -> Vec<(String, String)> ;

    fn report_mongodb_url(&self) -> Result<&str, Error>;
    
    fn case_batch_size(&self) -> usize;
}


#[derive(Component)]
#[shaku(interface = Config)]
#[derive(Debug, Clone)]
pub struct ConfigImpl {
    conf: Json
}



static mut SINGLETON: Option<ConfigImpl> = Option::None;

impl ConfigImpl{

    pub fn new(conf: Json) -> ConfigImpl{
        ConfigImpl {
            conf
        }
    }

    // pub fn create_singleton(conf: Json) -> &'static ConfigImpl{
    //     unsafe {
    //         SINGLETON = Some(ConfigImpl::new(conf));
    //         ConfigImpl::get_singleton()
    //     }
    // }

    pub fn get_singleton() -> &'static ConfigImpl{
        unsafe {&SINGLETON.as_ref().unwrap()}
    }
}

unsafe impl Send for ConfigImpl
{
}

unsafe impl Sync for ConfigImpl
{
}


impl Config for ConfigImpl {

    fn server_ip(&self) -> &str {
        self.conf["server"]["ip"].as_str().unwrap_or("127.0.0.1")
    }

    fn server_port(&self) -> usize {
        self.conf["server"]["port"].as_u64().unwrap_or(9999) as usize
    }

    fn log_path(&self) -> &str {
        self.conf["log"]["path"].as_str().unwrap_or("/data/chord/job/output/web.log")
    }

    fn job_input_path(&self) -> &str {
        self.conf["job"]["input"]["path"].as_str().unwrap_or("/data/chord/job/input")
    }

    fn ssh_key_private_path(&self) -> &str {
        self.conf["ssh"]["key"]["private"]["path"].as_str().unwrap_or("/data/chord/conf/ssh_key.pri")
    }

    fn log_level(&self) -> Vec<(String, String)>{
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

    fn report_mongodb_url(&self) -> Result<&str, Error> {
        self.conf["report"]["mongodb"]["url"].as_str().ok_or(err!("config", "missing report.mongodb.url"))
    }

    fn case_batch_size(&self) -> usize{
        self.conf["case"]["batch"]["size"].as_u64().unwrap_or(99999) as usize
    }
}
