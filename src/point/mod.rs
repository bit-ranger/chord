use crate::model::{PointContext};
use crate::model::PointResult;
mod restapi;
mod md5;

async fn run_point_type(point_type: &str, context: &PointContext<'_,'_>) ->  PointResult
{
    return if point_type.trim().eq("restapi") {
        restapi::run_point(context).await
    }else if point_type.trim().eq("md5") {
        md5::run_point(context).await
    } else {
        PointResult::Err(())
    }
}

pub async fn run_point(context: &PointContext<'_, '_>) -> PointResult
{
    let point_type = context.get_meta_str(vec!["type"]).await.unwrap();
    let result = run_point_type(point_type.as_str(), context).await;

    if result.is_err() {
        return PointResult::Err(());
    }

    let value = result.unwrap();
    let assert_condition = context.get_meta_str(vec!["assert"]).await;
    match assert_condition{
        Some(con) =>  {
            let assert_result = context.assert(con.as_str(), &value).await;
            if assert_result {PointResult::Ok(value)} else {PointResult::Err(())}
        },
        None => return Ok(value)
    }
}

