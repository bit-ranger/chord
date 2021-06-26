use std::collections::HashMap;

use chord::action::{Action, ActionFactory, CreateArg};
use chord::err;
use chord::step::async_trait;
use chord::value::Value;
use chord::Error;

mod echo;
mod log;
mod sleep;

#[cfg(feature = "act_crypto")]
mod crypto;
#[cfg(feature = "act_database")]
mod database;
#[cfg(feature = "act_docker")]
mod docker;
#[cfg(feature = "act_dubbo")]
mod dubbo;
#[cfg(feature = "act_dylib")]
mod dylib;
#[cfg(feature = "act_mongodb")]
mod mongodb;
#[cfg(feature = "act_redis")]
mod redis;
#[cfg(feature = "act_restapi")]
mod restapi;
#[cfg(feature = "act_url")]
mod url;

pub struct ActionFactoryDefault {
    table: HashMap<String, Box<dyn ActionFactory>>,
}

macro_rules! register {
    ($table:ident, $config_ref:ident, $name:expr, $module:path, $enable:expr) => {
        if enable($config_ref, $name, $enable) {
            $table.insert(
                $name.into(),
                Box::new($module($config_ref.map(|c| c[$name].clone())).await?),
            );
        }
    };
}

impl ActionFactoryDefault {
    pub async fn new(config: Option<Value>) -> Result<ActionFactoryDefault, Error> {
        let mut table: HashMap<String, Box<dyn ActionFactory>> = HashMap::new();

        let config_ref = config.as_ref();

        register!(table, config_ref, "echo", echo::Factory::new, true);
        register!(table, config_ref, "sleep", sleep::Factory::new, true);
        register!(table, config_ref, "log", log::Factory::new, true);

        #[cfg(feature = "act_restapi")]
        register!(table, config_ref, "restapi", restapi::Factory::new, true);

        #[cfg(feature = "act_crypto")]
        register!(table, config_ref, "crypto", crypto::Factory::new, true);

        #[cfg(feature = "act_url")]
        register!(table, config_ref, "url", url::Factory::new, true);

        #[cfg(feature = "act_dubbo")]
        register!(table, config_ref, "dubbo", dubbo::Factory::new, false);

        #[cfg(feature = "act_database")]
        register!(table, config_ref, "database", database::Factory::new, true);

        #[cfg(feature = "act_redis")]
        register!(table, config_ref, "redis", redis::Factory::new, true);

        #[cfg(feature = "act_mongodb")]
        register!(table, config_ref, "mongodb", mongodb::Factory::new, true);

        #[cfg(feature = "act_dylib")]
        register!(table, config_ref, "dylib", dylib::Factory::new, false);

        #[cfg(feature = "act_docker")]
        register!(table, config_ref, "docker", docker::Factory::new, true);

        Ok(ActionFactoryDefault { table })
    }
}

#[async_trait]
impl ActionFactory for ActionFactoryDefault {
    async fn create(&self, arg: &dyn CreateArg) -> Result<Box<dyn Action>, Error> {
        let action = arg.action();
        self.table
            .get(action)
            .ok_or(err!(
                "002",
                format!("unsupported step action {}", action).as_str()
            ))?
            .create(arg)
            .await
    }
}

fn enable(config: Option<&Value>, step_name: &str, default_enable: bool) -> bool {
    if config.is_none() {
        return default_enable;
    }
    let config_ref = config.unwrap();
    if config_ref.is_null() {
        return default_enable;
    }

    return config_ref[step_name]["enable"]
        .as_bool()
        .unwrap_or(default_enable);
}
