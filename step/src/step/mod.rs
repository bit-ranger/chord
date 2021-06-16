use std::collections::HashMap;

use chord::err;
use chord::step::{async_trait, CreateArg, StepRunner, StepRunnerFactory};
use chord::value::Value;
use chord::Error;

mod echo;
mod log;
mod sleep;

#[cfg(feature = "step_crypto")]
mod crypto;
#[cfg(feature = "step_database")]
mod database;
#[cfg(feature = "step_dubbo")]
mod dubbo;
#[cfg(feature = "step_dylib")]
mod dylib;
#[cfg(feature = "step_mongodb")]
mod mongodb;
#[cfg(feature = "step_redis")]
mod redis;
#[cfg(feature = "step_restapi")]
mod restapi;
#[cfg(feature = "step_url")]
mod url;

pub struct StepRunnerFactoryDefault {
    table: HashMap<String, Box<dyn StepRunnerFactory>>,
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

impl StepRunnerFactoryDefault {
    pub async fn new(config: Option<Value>) -> Result<StepRunnerFactoryDefault, Error> {
        let mut table: HashMap<String, Box<dyn StepRunnerFactory>> = HashMap::new();

        let config_ref = config.as_ref();

        register!(table, config_ref, "echo", echo::Factory::new, true);
        register!(table, config_ref, "sleep", sleep::Factory::new, true);
        register!(table, config_ref, "log", log::Factory::new, true);

        #[cfg(feature = "step_restapi")]
        register!(table, config_ref, "restapi", restapi::Factory::new, true);

        #[cfg(feature = "step_crypto")]
        register!(table, config_ref, "crypto", crypto::Factory::new, true);

        #[cfg(feature = "step_url")]
        register!(table, config_ref, "url", url::Factory::new, true);

        #[cfg(feature = "step_dubbo")]
        register!(table, config_ref, "dubbo", dubbo::Factory::new, false);

        #[cfg(feature = "step_database")]
        register!(table, config_ref, "database", database::Factory::new, true);

        #[cfg(feature = "step_redis")]
        register!(table, config_ref, "redis", redis::Factory::new, true);

        #[cfg(feature = "step_mongodb")]
        register!(table, config_ref, "mongodb", mongodb::Factory::new, true);

        #[cfg(feature = "step_dylib")]
        register!(table, config_ref, "dylib", dylib::Factory::new, false);

        Ok(StepRunnerFactoryDefault { table })
    }
}

#[async_trait]
impl StepRunnerFactory for StepRunnerFactoryDefault {
    async fn create(&self, arg: &dyn CreateArg) -> Result<Box<dyn StepRunner>, Error> {
        let kind = arg.kind();
        self.table
            .get(kind)
            .ok_or(err!(
                "002",
                format!("unsupported step kind {}", kind).as_str()
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
