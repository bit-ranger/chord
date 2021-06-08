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
#[cfg(feature = "step_jsonapi")]
mod jsonapi;
#[cfg(feature = "step_md5")]
mod md5;
#[cfg(feature = "step_mongodb")]
mod mongodb;
#[cfg(feature = "step_mysql")]
mod mysql;
#[cfg(feature = "step_redis")]
mod redis;
#[cfg(feature = "step_url_encode")]
mod url_decode;
#[cfg(feature = "step_url_encode")]
mod url_encode;

pub struct StepRunnerFactoryDefault {
    table: HashMap<String, Box<dyn StepRunnerFactory>>,
}

impl StepRunnerFactoryDefault {
    pub async fn new(config: Option<Json>) -> Result<StepRunnerFactoryDefault, Error> {
        let mut table: HashMap<String, Box<dyn StepRunnerFactory>> = HashMap::new();

        let config_ref = config.as_ref();
        table.insert(
            "sleep".into(),
            Box::new(sleep::Factory::new(config_ref.map(|c| c["sleep"].clone())).await?),
        );

        #[cfg(feature = "step_jsonapi")]
        if enable(config_ref, "jsonapi") {
            table.insert(
                "jsonapi".into(),
                Box::new(jsonapi::Factory::new(config_ref.map(|c| c["jsonapi"].clone())).await?),
            );
        }

        #[cfg(feature = "step_md5")]
        if enable(config_ref, "md5") {
            table.insert(
                "md5".into(),
                Box::new(md5::Factory::new(config_ref.map(|c| c["md5"].clone())).await?),
            );
        }

        #[cfg(feature = "step_url_encode")]
        if enable(config_ref, "url_encode") {
            table.insert(
                "url_encode".into(),
                Box::new(
                    url_encode::Factory::new(config_ref.map(|c| c["url_encode"].clone())).await?,
                ),
            );
        }

        #[cfg(feature = "step_url_decode")]
        if enable(config_ref, "url_decode") {
            table.insert(
                "url_decode".into(),
                Box::new(
                    url_decode::Factory::new(config_ref.map(|c| c["url_decode"].clone())).await?,
                ),
            );
        }

        #[cfg(feature = "step_dubbo")]
        if enable(config_ref, "dubbo") {
            table.insert(
                "dubbo".into(),
                Box::new(dubbo::Factory::new(config_ref.map(|c| c["dubbo"].clone())).await?),
            );
        }

        #[cfg(feature = "step_mysql")]
        if enable(config_ref, "mysql") {
            table.insert(
                "mysql".into(),
                Box::new(mysql::Factory::new(config_ref.map(|c| c["mysql"].clone())).await?),
            );
        }

        #[cfg(feature = "step_redis")]
        if enable(config_ref, "redis") {
            table.insert(
                "redis".into(),
                Box::new(redis::Factory::new(config_ref.map(|c| c["redis"].clone())).await?),
            );
        }

        #[cfg(feature = "step_mongodb")]
        if enable(config_ref, "mongodb") {
            table.insert(
                "mongodb".into(),
                Box::new(mongodb::Factory::new(config_ref.map(|c| c["mongodb"].clone())).await?),
            );
        }

        #[cfg(feature = "step_dynlib")]
        if enable(config_ref, "dynlib") {
            table.insert(
                "dynlib".into(),
                Box::new(dynlib::Factory::new(config_ref.map(|c| c["dynlib"].clone())).await?),
            );
        }

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

fn enable(config: Option<&Json>, step_name: &str) -> bool {
    if config.is_none() {
        return true;
    }
    let config_ref = config.unwrap();
    if config_ref.is_null() {
        return true;
    }

    return config_ref[step_name]["enable"].as_bool().unwrap_or(true);
}
