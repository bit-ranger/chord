mod model;
mod point;

use std::future::Future;
use std::pin::Pin;

use common::point::{PointArg, PointRunner};

#[macro_export]
macro_rules! err {
    ($code:expr, $message:expr) => {{
        let res = $crate::model::PointError::new($code, $message);
        std::result::Result::Err(res)
    }}
}

#[macro_export]
macro_rules! perr {
    ($code:expr, $message:expr) => {{
        $crate::model::PointError::new($code, $message)
    }}
}


pub struct PointRunnerDefault;

impl PointRunnerDefault{
    pub fn new() -> PointRunnerDefault{
        PointRunnerDefault{}
    }
}

impl PointRunner for PointRunnerDefault {

    fn run<'a>(&self, kind: &'a str, arg: &'a dyn PointArg) -> Pin<Box<dyn Future<Output=common::point::PointValue> + Send  + 'a>> {
        Box::pin(crate::run_point_kind(kind, arg))
    }
}


unsafe impl Send for PointRunnerDefault
{
}

unsafe impl Sync for PointRunnerDefault
{
}


async fn run_point_kind(kind: &str, arg: &dyn PointArg) -> common::point::PointValue{

    let value = match kind.trim() {
        "sleep" => point::sleep::run(arg).await,
        #[cfg(feature = "pt_restapi")]
        "restapi" => point::restapi::run(arg).await,
        #[cfg(feature = "pt_md5")]
        "md5" => point::md5::run(arg).await,
        #[cfg(feature = "pt_dubbo")]
        "dubbo" => point::dubbo::run(arg).await,
        #[cfg(feature = "pt_mysql")]
        "mysql" => point::mysql::run(arg).await,
        #[cfg(feature = "pt_redis")]
        "redis" => point::redis::run(arg).await,
        _ => err!("002", format!("unsupported point kind {}", kind).as_str())
    };

    return model::to_common_value(value);
}





