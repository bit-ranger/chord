use crate::model::{PointContext, PointResult, Error};

mod restapi;
mod md5;


pub async fn run_point_type(point_type: &str, context: &dyn PointContext) ->  PointResult
{
    return if point_type.trim().eq("restapi") {
        restapi::run_point(context).await
    }else if point_type.trim().eq("md5") {
        md5::run_point(context).await
    } else {
        PointResult::Err(Error::new("002", format!("unsupported point type {}", point_type).as_str()))
    }
}