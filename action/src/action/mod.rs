use std::collections::HashMap;
use std::sync::Arc;

use chord_core::action::prelude::*;

use crate::err;

mod assert;
mod count;
mod echo;
mod iter;
mod log;
mod nop;
mod sleep;

#[cfg(feature = "act_cdylib")]
mod cdylib;
#[cfg(feature = "act_crypto")]
mod crypto;
#[cfg(feature = "act_database")]
mod database;
#[cfg(feature = "act_docker")]
mod docker;
#[cfg(feature = "act_dubbo")]
mod dubbo;
#[cfg(feature = "act_lua")]
mod lua;
#[cfg(feature = "act_mongodb")]
mod mongodb;
#[cfg(feature = "act_program")]
mod program;
#[cfg(feature = "act_redis")]
mod redis;
#[cfg(feature = "act_restapi")]
mod restapi;
#[cfg(feature = "act_url")]
mod url;

pub struct FactoryComposite {
    table: HashMap<String, Arc<dyn Factory>>,
}

macro_rules! register {
    ($table:ident, $config_ref:ident, $name:expr, $module:path) => {
        if enable($config_ref, $name) {
            $table.insert(
                $name.into(),
                Arc::new($module($config_ref.map(|c| c[$name].clone())).await?),
            );
        }
    };
}

impl FactoryComposite {
    pub async fn new(config: Option<Value>) -> Result<FactoryComposite, Error> {
        let mut table: HashMap<String, Arc<dyn Factory>> = HashMap::new();

        let config_ref = config.as_ref();

        register!(table, config_ref, "assert", assert::AssertFactory::new);

        register!(table, config_ref, "nop", nop::NopFactory::new);
        register!(table, config_ref, "echo", echo::EchoFactory::new);
        register!(table, config_ref, "sleep", sleep::SleepFactory::new);
        register!(table, config_ref, "log", log::LogFactory::new);
        register!(table, config_ref, "count", count::CountFactory::new);

        #[cfg(feature = "act_restapi")]
        register!(table, config_ref, "restapi", restapi::RestapiFactory::new);

        #[cfg(feature = "act_crypto")]
        register!(table, config_ref, "crypto", crypto::CryptoFactory::new);

        #[cfg(feature = "act_url")]
        register!(table, config_ref, "url", url::UrlFactory::new);

        #[cfg(feature = "act_database")]
        register!(
            table,
            config_ref,
            "database",
            database::DatabaseFactory::new
        );

        #[cfg(feature = "act_redis")]
        register!(table, config_ref, "redis", redis::RedisFactory::new);

        #[cfg(feature = "act_mongodb")]
        register!(table, config_ref, "mongodb", mongodb::MongodbFactory::new);

        #[cfg(feature = "act_lua")]
        register!(table, config_ref, "lua", lua::LuaFactory::new);

        #[cfg(feature = "act_program")]
        register!(table, config_ref, "program", program::ProgramFactory::new);

        #[cfg(feature = "act_dubbo")]
        register!(table, config_ref, "dubbo", dubbo::DubboFactory::new);

        #[cfg(feature = "act_cdylib")]
        register!(table, config_ref, "cdylib", cdylib::CdylibFactory::new);

        #[cfg(feature = "act_docker")]
        register!(table, config_ref, "docker", docker::Docker::new);

        #[cfg(feature = "act_download")]
        register!(
            table,
            config_ref,
            "download",
            download::DownloadFactory::new
        );

        if enable(config_ref, "iter_map") {
            table.insert(
                "iter_map".into(),
                Arc::new(
                    iter::map::IterMapFactory::new(
                        config_ref.map(|c| c["iter_map"].clone()),
                        table.clone(),
                    )
                    .await?,
                ),
            );
        }

        Ok(FactoryComposite { table })
    }
}

#[async_trait]
impl Factory for FactoryComposite {
    async fn create(&self, arg: &dyn CreateArg) -> Result<Box<dyn Action>, Error> {
        let action = arg.action();
        self.table
            .get(action)
            .ok_or(err!(
                "002",
                format!("unsupported action {}", action).as_str()
            ))?
            .create(arg)
            .await
    }
}

fn enable(config: Option<&Value>, action_name: &str) -> bool {
    let default_enable = true;
    if config.is_none() {
        return default_enable;
    }
    let config_ref = config.unwrap();
    if config_ref.is_null() {
        return default_enable;
    }

    return config_ref[action_name]["enable"]
        .as_bool()
        .unwrap_or(default_enable);
}
