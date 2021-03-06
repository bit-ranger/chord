use crate::model::{Error, PointResult};
use crate::flow::point::model::{PointContextStruct};
use crate::point::{run_point_type};

pub mod model;




pub async fn run_point(context: &PointContextStruct<'_, '_, '_, '_, '_>) -> PointResult
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

