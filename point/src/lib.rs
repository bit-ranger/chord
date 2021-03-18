mod model;
mod ext;

use std::future::Future;
use std::pin::Pin;

use common::point::{PointArg, PointRunner};

#[macro_export]
macro_rules! err {
    ($code:expr, $message:expr) => {{
        let res = $crate::model::Error::new($code, $message);
        std::result::Result::Err(res)
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


async fn run_point_type(point_type: &str, context: &dyn PointArg) -> common::point::PointValue{

    let point_value = match point_type.trim() {
        "restapi" => ext::restapi::run(context).await,
        "md5" => ext::md5::run(context).await,
        _ => err!("002", format!("unsupported point type {}", point_type).as_str())
    };

    return model::to_common_value(point_value);
}





