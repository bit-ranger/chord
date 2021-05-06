use chord_common::rerr;
use chord_common::point::{PointRunner};
use chord_common::error::Error;
use chord_common::value::Json;

pub mod sleep;

#[cfg(feature = "pt_jsonapi")]
pub mod jsonapi;
#[cfg(feature = "pt_md5")]
pub mod md5;
#[cfg(feature = "pt_dubbo")]
pub mod dubbo;
#[cfg(feature = "pt_mysql")]
pub mod mysql;
#[cfg(feature = "pt_redis")]
pub mod redis;
#[cfg(feature = "pt_mongodb")]
pub mod mongodb;

pub async fn create_kind_runner(kind: &str, config: &Json) -> Result<Box<dyn PointRunner>, Error>{

    let value = match kind.trim() {
        "sleep" => sleep::create(config).await,

        #[cfg(feature = "pt_jsonapi")]
        "jsonapi" => jsonapi::create(config).await,
        #[cfg(feature = "pt_md5")]
        "md5" => md5::create(config).await,
        #[cfg(feature = "pt_dubbo")]
        "dubbo" => dubbo::create(config).await,
        #[cfg(feature = "pt_mysql")]
        "mysql" => mysql::create(config).await,
        #[cfg(feature = "pt_redis")]
        "redis" => redis::create(config).await,
        #[cfg(feature = "pt_mongodb")]
        "mongodb" => mongodb::create(config).await,
        _ => rerr!("002", format!("unsupported point kind {}", kind).as_str())
    };

    return value;
}
