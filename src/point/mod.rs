use crate::model::point::{PointArg, PointRunner};
use crate::model::error::Error;
use crate::model::point::PointValue;
use futures::Future;
use std::pin::Pin;

mod restapi;
mod md5;


pub struct PointRunnerDefault;

impl PointRunnerDefault{
    pub fn new() -> PointRunnerDefault{
        PointRunnerDefault{}
    }
}

impl PointRunner for PointRunnerDefault {

    fn run<'a>(&self, point_type: &'a str, context: &'a dyn PointArg) -> Pin<Box<dyn Future<Output=PointValue> + 'a>> {
        Box::pin(run_point_type(point_type, context))
    }
}

async fn run_point_type(point_type: &str, context: &dyn PointArg) -> PointValue{
    return if point_type.trim().eq("restapi") {
        restapi::run(context).await
    }else if point_type.trim().eq("md5") {
        md5::run(context).await
    } else {
        PointValue::Err(Error::new("002", format!("unsupported point type {}", point_type).as_str()))
    }
}