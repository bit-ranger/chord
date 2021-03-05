use crate::model::point::{PointContextStruct, PointContext};
use crate::model::point::PointResult;
use crate::model::Error;

mod restapi;
mod md5;

async fn run_point_type(point_type: &str, context: &dyn PointContext) ->  PointResult
{
    return if point_type.trim().eq("restapi") {
        restapi::run_point(context).await
    }else if point_type.trim().eq("md5") {
        md5::run_point(context).await
    } else {
        PointResult::Err(Error::new("002", format!("unsupported point type {}", point_type).as_str()))
    }
}

pub async fn run_point(context: &PointContextStruct<'_, '_>) -> PointResult
{
    let point_type = context.get_meta_str(vec!["type"]).await.unwrap();
    let result = run_point_type(point_type.as_str(), context).await;

    if result.is_err() {
        return PointResult::Err(Error::new("000", "run point failure"));
    }

    let value = result.unwrap();
    let assert_condition = context.get_meta_str(vec!["assert"]).await;
    match assert_condition{
        Some(con) =>  {
            let assert_result = context.assert(con.as_str(), &value).await;
            if assert_result {PointResult::Ok(value)} else {PointResult::Err(Error::new("001", "assert point failure"))}
        },
        None => return Ok(value)
    }
}

