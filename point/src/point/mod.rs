use chord_common::point::PointArg;

use crate::point;

pub mod sleep;

#[cfg(feature = "pt_restapi")]
pub mod restapi;
#[cfg(feature = "pt_md5")]
pub mod md5;
#[cfg(feature = "pt_dubbo")]
pub mod dubbo;
#[cfg(feature = "pt_mysql")]
pub mod mysql;
#[cfg(feature = "pt_redis")]
pub mod redis;


pub async fn run_point_kind(kind: &str, arg: &dyn PointArg) -> chord_common::point::PointValue{

    let value = match kind.trim() {
        "sleep" => sleep::run(arg).await,

        #[cfg(feature = "pt_restapi")]
        "restapi" => restapi::run(arg).await,
        #[cfg(feature = "pt_md5")]
        "md5" => md5::run(arg).await,
        #[cfg(feature = "pt_dubbo")]
        "dubbo" => dubbo::run(arg).await,
        #[cfg(feature = "pt_mysql")]
        "mysql" => mysql::run(arg).await,
        #[cfg(feature = "pt_redis")]
        "redis" => redis::run(arg).await,

        _ => err!("002", format!("unsupported point kind {}", kind).as_str())
    };

    return value;
}
