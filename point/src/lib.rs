mod model;
mod ext;

use std::future::Future;
use std::pin::Pin;
use log::{info, error};

use common::point::{PointArg, PointRunner};

#[macro_export]
macro_rules! err {
    ($code:expr, $message:expr) => {{
        let res = $crate::model::PointError::new($code, $message);
        std::result::Result::Err(res)
    }}
}

#[macro_export]
macro_rules! err_raw {
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
        "restapi" => ext::restapi::run(point_arg).await,
        "md5" => ext::md5::run(point_arg).await,
        "dubbo" => ext::dubbo::run(point_arg).await,
        "sleep" => ext::sleep::run(point_arg).await,
        _ => err!("002", format!("unsupported point type {}", point_type).as_str())
    };

    let point_value = model::to_common_value(point_value);
    let config_text = point_arg.config();
    let config_text = point_arg.render(&format!("{}", config_text));
    match &point_value {
        Ok(pv) =>   {
            info!("PointValue: {} - OK  - {} \n>>> {}", point_type, pv, config_text.unwrap());
        },
        Err(e) => {
            error!("PointValue: {} - ERR - {} \n>>> {}", point_type, e,  point_arg.config())
        }
    }

    return point_value;
}





