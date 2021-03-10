use crate::flow::point::model::PointContextStruct;
use crate::model::error::Error;
use crate::model::context::PointResult;
use crate::point::run_point_type;
use crate::model::value::Json;

pub mod model;




pub async fn run_point(context: &PointContextStruct<'_, '_, '_, '_, '_>) -> PointResult
{
    let point_type = context.get_meta_str(vec!["type"]).await;
    if point_type.is_none(){
        return PointResult::Err(Error::new("001", "missing type"));
    }
    let point_type = point_type.unwrap();

    return run_point_type(point_type.as_str(), context).await;
}

pub async fn assert(context: &PointContextStruct<'_, '_, '_, '_, '_>, result: &Json) -> bool{
    let assert_condition = context.get_meta_str(vec!["assert"]).await;
    return match assert_condition{
        Some(con) =>  {
            context.assert(con.as_str(), &result).await
        },
        None => true
    }
}

