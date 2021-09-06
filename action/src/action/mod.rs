use std::collections::HashMap;

use async_std::sync::Arc;
use chord::action::prelude::*;
use chord::err;

mod count;
mod echo;
mod iter;
mod log;
mod sleep;

#[cfg(feature = "act_crypto")]
mod crypto;
#[cfg(feature = "act_database")]
mod database;
#[cfg(feature = "act_docker")]
mod docker;
#[cfg(feature = "act_download")]
mod download;
#[cfg(feature = "act_dubbo")]
mod dubbo;
#[cfg(feature = "act_dylib")]
mod dylib;
#[cfg(feature = "act_fstore")]
mod fstore;
#[cfg(feature = "act_lua")]
mod lua;
#[cfg(feature = "act_mongodb")]
mod mongodb;
#[cfg(feature = "act_redis")]
mod redis;
#[cfg(feature = "act_restapi")]
mod restapi;
#[cfg(all(feature = "act_shell", target_os = "linux"))]
mod shell;
#[cfg(feature = "act_url")]
mod url;

pub struct FactoryComposite {
    table: HashMap<String, Arc<dyn Factory>>,
}

macro_rules! register {
    ($table:ident, $config_ref:ident, $name:expr, $module:path, $enable:expr) => {
        if enable($config_ref, $name, $enable) {
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

        register!(table, config_ref, "echo", echo::EchoFactory::new, true);
        register!(table, config_ref, "sleep", sleep::SleepFactory::new, true);
        register!(table, config_ref, "log", log::LogFactory::new, true);
        register!(table, config_ref, "count", count::CountFactory::new, true);

        #[cfg(feature = "act_restapi")]
        register!(
            table,
            config_ref,
            "restapi",
            restapi::RestapiFactory::new,
            true
        );

        #[cfg(feature = "act_crypto")]
        register!(
            table,
            config_ref,
            "crypto",
            crypto::CryptoFactory::new,
            true
        );

        #[cfg(feature = "act_url")]
        register!(table, config_ref, "url", url::UrlFactory::new, true);

        #[cfg(feature = "act_database")]
        register!(
            table,
            config_ref,
            "database",
            database::DatabaseFactory::new,
            true
        );

        #[cfg(feature = "act_redis")]
        register!(table, config_ref, "redis", redis::RedisFactory::new, true);

        #[cfg(feature = "act_mongodb")]
        register!(
            table,
            config_ref,
            "mongodb",
            mongodb::MongodbFactory::new,
            true
        );

        #[cfg(feature = "act_lua")]
        register!(table, config_ref, "lua", lua::LuaFactory::new, true);

        #[cfg(feature = "act_dubbo")]
        register!(table, config_ref, "dubbo", dubbo::DubboFactory::new, false);

        #[cfg(feature = "act_dylib")]
        register!(table, config_ref, "dylib", dylib::DylibFactory::new, false);

        #[cfg(feature = "act_docker")]
        register!(table, config_ref, "docker", docker::Docker::new, false);

        #[cfg(feature = "act_download")]
        register!(
            table,
            config_ref,
            "download",
            download::DownloadFactory::new,
            false
        );

        #[cfg(feature = "act_fstore")]
        register!(
            table,
            config_ref,
            "fstore",
            fstore::FstoreFactory::new,
            false
        );

        #[cfg(all(feature = "act_shell", target_os = "linux"))]
        register!(table, config_ref, "shell", shell::ShellFactory::new, false);

        register!(
            table,
            config_ref,
            "iter_filter",
            iter::filter::IterFilterFactory::new,
            true
        );

        register!(
            table,
            config_ref,
            "iter_flatten",
            iter::flatten::IterFlattenFactory::new,
            true
        );

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
