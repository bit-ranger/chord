use std::future::Future;
use std::pin::Pin;

use chord_common::point::{PointArg, PointRunner};

mod point;

use chord_common::{err};

pub struct PointRunnerDefault;

impl PointRunnerDefault{
    pub fn new() -> PointRunnerDefault{
        PointRunnerDefault{}
    }
}

impl PointRunner for PointRunnerDefault {

    fn run<'a>(&self, kind: &'a str, arg: &'a dyn PointArg) -> Pin<Box<dyn Future<Output=chord_common::point::PointValue> + Send  + 'a>> {
        Box::pin(crate::run_point_kind(kind, arg))
    }
}


unsafe impl Send for PointRunnerDefault
{
}

unsafe impl Sync for PointRunnerDefault
{
}


async fn run_point_kind(kind: &str, arg: &dyn PointArg) -> chord_common::point::PointValue{

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

    return value;
}





