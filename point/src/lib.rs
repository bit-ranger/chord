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

    fn run<'a>(&self, point_type: &'a str, context: &'a dyn PointArg) -> Pin<Box<dyn Future<Output=common::point::PointValue> + 'a>> {
        Box::pin(crate::run_point_type(point_type, context))
    }
}


async fn run_point_type(point_type: &str, point_arg: &dyn PointArg) -> common::point::PointValue{

    let point_value = match point_type.trim() {
        "restapi" => point::restapi::run(point_arg).await,
        "md5" => point::md5::run(point_arg).await,
        "dubbo" => point::dubbo::run(point_arg).await,
        "sleep" => point::sleep::run(point_arg).await,
        "mysql" => point::mysql::run(point_arg).await,
        _ => err!("002", format!("unsupported point type {}", point_type).as_str())
    };

    return model::to_common_value(point_value);
}





