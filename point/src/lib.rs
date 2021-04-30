use std::future::Future;
use std::pin::Pin;

use chord_common::point::{PointArg, PointRunner};
use chord_common::error::Error;

mod point;

pub struct PointRunnerDefault;

impl PointRunnerDefault{
    pub async fn new() -> Result<PointRunnerDefault, Error>{
        Ok(PointRunnerDefault{})
    }
}

impl PointRunner for PointRunnerDefault {

    fn run<'a>(&self, kind: &'a str, arg: &'a dyn PointArg) -> Pin<Box<dyn Future<Output=chord_common::point::PointValue> + Send  + 'a>> {
        Box::pin(point::run_point_kind(kind, arg))
    }
}


unsafe impl Send for PointRunnerDefault
{
}

unsafe impl Sync for PointRunnerDefault
{
}





