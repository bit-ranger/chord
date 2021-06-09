use chord_common::err;
use chord_common::error::Error;
use chord_common::step::{async_trait, CreateArg, StepRunner, StepRunnerFactory};
use chord_common::value::Json;
use std::collections::HashMap;

mod sleep;

#[cfg(feature = "step_dubbo")]
mod dubbo;
#[cfg(feature = "step_dynlib")]
mod dynlib;
#[cfg(feature = "step_md5")]
mod md5;
#[cfg(feature = "step_mongodb")]
mod mongodb;
#[cfg(feature = "step_mysql")]
mod mysql;
#[cfg(feature = "step_redis")]
mod redis;
#[cfg(feature = "step_restapi")]
mod restapi;
#[cfg(feature = "step_url_encode")]
mod url_decode;
#[cfg(feature = "step_url_encode")]
mod url_encode;

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
    pub async fn new(config: Option<Json>) -> Result<StepRunnerFactoryDefault, Error> {
        let mut table: HashMap<String, Box<dyn StepRunnerFactory>> = HashMap::new();

        let config_ref = config.as_ref();

        register!(table, config_ref, "sleep", sleep::Factory::new, true);

        #[cfg(feature = "step_restapi")]
        register!(table, config_ref, "restapi", restapi::Factory::new, true);

        #[cfg(feature = "step_md5")]
        register!(table, config_ref, "md5", md5::Factory::new, true);

        #[cfg(feature = "step_url_encode")]
        register!(
            table,
            config_ref,
            "url_encode",
            url_encode::Factory::new,
            true
        );

        #[cfg(feature = "step_url_decode")]
        register!(
            table,
            config_ref,
            "url_decode",
            url_decode::Factory::new,
            true
        );

        #[cfg(feature = "step_dubbo")]
        register!(table, config_ref, "dubbo", dubbo::Factory::new, false);

        #[cfg(feature = "step_mysql")]
        register!(table, config_ref, "mysql", mysql::Factory::new, true);

        #[cfg(feature = "step_redis")]
        register!(table, config_ref, "redis", redis::Factory::new, true);

        #[cfg(feature = "step_mongodb")]
        register!(table, config_ref, "mongodb", mongodb::Factory::new, true);

        #[cfg(feature = "step_dynlib")]
        register!(table, config_ref, "dynlib", dynlib::Factory::new, false);

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

fn enable(config: Option<&Json>, step_name: &str, default_enable: bool) -> bool {
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
