use crate::model::point::PointArg;
use crate::model::error::Error;
use crate::model::point::PointValue;

mod restapi;
mod md5;


pub async fn run_point_type(point_type: &str, context: &dyn PointArg) -> PointValue
{
    return if point_type.trim().eq("restapi") {
        restapi::run(context).await
    }else if point_type.trim().eq("md5") {
        md5::run(context).await
    } else {
        PointValue::Err(Error::new("002", format!("unsupported point type {}", point_type).as_str()))
    }
}