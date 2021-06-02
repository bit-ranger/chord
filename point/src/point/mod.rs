use chord_common::error::Error;
use chord_common::point::{PointRunner, CreateArg};
use chord_common::rerr;
use chord_common::value::Json;

pub mod sleep;

#[cfg(feature = "point_dubbo")]
pub mod dubbo;
#[cfg(feature = "point_dynlib")]
pub mod dynlib;
#[cfg(feature = "point_jsonapi")]
pub mod jsonapi;
#[cfg(feature = "point_md5")]
pub mod md5;
#[cfg(feature = "point_mongodb")]
pub mod mongodb;
#[cfg(feature = "point_mysql")]
pub mod mysql;
#[cfg(feature = "point_redis")]
pub mod redis;
#[cfg(feature = "point_url_encode")]
pub mod url_decode;
#[cfg(feature = "point_url_encode")]
pub mod url_encode;

pub async fn create_kind_runner(
    kind: &str,
    config: Option<&Json>,
    arg: &dyn CreateArg,
) -> Result<Box<dyn PointRunner>, Error> {
    let value = match kind.trim() {
        "sleep" => sleep::create(config, arg).await,

        #[cfg(feature = "point_jsonapi")]
        "jsonapi" => jsonapi::create(config, arg).await,
        #[cfg(feature = "point_md5")]
        "md5" => md5::create(config, arg).await,
        #[cfg(feature = "point_url_encode")]
        "url_encode" => url_encode::create(config, arg).await,
        #[cfg(feature = "point_url_decode")]
        "url_decode" => url_decode::create(config, arg).await,
        #[cfg(feature = "point_dubbo")]
        "dubbo" => dubbo::create(config, arg).await,
        #[cfg(feature = "point_mysql")]
        "mysql" => mysql::create(config, arg).await,
        #[cfg(feature = "point_redis")]
        "redis" => redis::create(config, arg).await,
        #[cfg(feature = "point_mongodb")]
        "mongodb" => mongodb::create(config, arg).await,
        #[cfg(feature = "point_dynlib")]
        "dynlib" => dynlib::create(config, arg).await,
        _ => rerr!("002", format!("unsupported point kind {}", kind).as_str()),
    };

    return value;
}
