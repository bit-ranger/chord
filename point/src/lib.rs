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

    fn run<'a>(&self, pt_type: &'a str, context: &'a dyn PointArg) -> Pin<Box<dyn Future<Output=common::point::PointValue> + 'a>> {
        Box::pin(crate::run_pt_type(pt_type, context))
    }
}


async fn run_pt_type(pt_type: &str, pt_arg: &dyn PointArg) -> common::point::PointValue{

    let pt_value = match pt_type.trim() {
        #[cfg(feature = "pt_restapi")]
        "restapi" => point::restapi::run(pt_arg).await,
        #[cfg(feature = "pt_md5")]
        "md5" => point::md5::run(pt_arg).await,
        #[cfg(feature = "pt_dubbo")]
        "dubbo" => point::dubbo::run(pt_arg).await,
        "sleep" => point::sleep::run(pt_arg).await,
        #[cfg(feature = "pt_mysql")]
        "mysql" => point::mysql::run(pt_arg).await,
        #[cfg(feature = "pt_redis")]
        "redis" => point::redis::run(pt_arg).await,
        _ => err!("002", format!("unsupported point type {}", pt_type).as_str())
    };

    return model::to_common_value(pt_value);
}





