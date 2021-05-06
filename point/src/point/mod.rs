use chord_common::rerr;
use chord_common::point::{PointRunner, PointArg};
use chord_common::error::Error;

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

pub async fn create_kind_runner(kind: &str, arg: &dyn PointArg) -> Result<Box<dyn PointRunner>, Error>{

    let value = match kind.trim() {
        "sleep" => sleep::create(arg).await,

        #[cfg(feature = "pt_jsonapi")]
        "jsonapi" => jsonapi::create(arg).await,
        #[cfg(feature = "pt_md5")]
        "md5" => md5::create(arg).await,
        #[cfg(feature = "pt_dubbo")]
        "dubbo" => dubbo::create(arg).await,
        #[cfg(feature = "pt_mysql")]
        "mysql" => mysql::create(arg).await,
        #[cfg(feature = "pt_redis")]
        "redis" => redis::create(arg).await,
        #[cfg(feature = "pt_mongodb")]
        "mongodb" => mongodb::create(arg).await,
        _ => rerr!("002", format!("unsupported point kind {}", kind).as_str())
    };

    return value;
}
