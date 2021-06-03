use chord_common::error::Error;
use chord_common::step::{StepRunner, CreateArg};
use chord_common::rerr;
use chord_common::value::Json;

pub mod sleep;

#[cfg(feature = "step_dubbo")]
pub mod dubbo;
#[cfg(feature = "step_dynlib")]
pub mod dynlib;
#[cfg(feature = "step_jsonapi")]
pub mod jsonapi;
#[cfg(feature = "step_md5")]
pub mod md5;
#[cfg(feature = "step_mongodb")]
pub mod mongodb;
#[cfg(feature = "step_mysql")]
pub mod mysql;
#[cfg(feature = "step_redis")]
pub mod redis;
#[cfg(feature = "step_url_encode")]
pub mod url_decode;
#[cfg(feature = "step_url_encode")]
pub mod url_encode;

pub async fn create_kind_runner(
    kind: &str,
    config: Option<&Json>,
    arg: &dyn CreateArg,
) -> Result<Box<dyn StepRunner>, Error> {
    let value = match kind.trim() {
        "sleep" => sleep::create(config, arg).await,

        #[cfg(feature = "step_jsonapi")]
        "jsonapi" => jsonapi::create(config, arg).await,
        #[cfg(feature = "step_md5")]
        "md5" => md5::create(config, arg).await,
        #[cfg(feature = "step_url_encode")]
        "url_encode" => url_encode::create(config, arg).await,
        #[cfg(feature = "step_url_decode")]
        "url_decode" => url_decode::create(config, arg).await,
        #[cfg(feature = "step_dubbo")]
        "dubbo" => dubbo::create(config, arg).await,
        #[cfg(feature = "step_mysql")]
        "mysql" => mysql::create(config, arg).await,
        #[cfg(feature = "step_redis")]
        "redis" => redis::create(config, arg).await,
        #[cfg(feature = "step_mongodb")]
        "mongodb" => mongodb::create(config, arg).await,
        #[cfg(feature = "step_dynlib")]
        "dynlib" => dynlib::create(config, arg).await,
        _ => rerr!("002", format!("unsupported step kind {}", kind).as_str()),
    };

    return value;
}
