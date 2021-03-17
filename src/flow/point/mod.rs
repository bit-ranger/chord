use crate::flow::point::model::PointContextStruct;
use crate::model::error::Error;
use crate::model::context::{PointResult, PointResultStruct, PointResultInner};
use crate::point::run_point_type;
use crate::model::value::Json;
use chrono::Utc;

pub mod model;




pub async fn run(context: &PointContextStruct<'_, '_, '_, '_, '_>) -> PointResultInner
{
    let start = Utc::now();
    let point_type = context.get_meta_str(vec!["type"]).await;
    if point_type.is_none(){
        return PointResultInner::Err(Error::new("001", "missing type"));
    }
    let point_type = point_type.unwrap();

    let result = run_point_type(point_type.as_str(), context).await;
    let end = Utc::now();

    return match result{
        PointResult::Ok(json) => {
            let result_struct = PointResultStruct::new(json, context.get_id(), start, end);
            PointResultInner::Ok(result_struct)
        },
        PointResult::Err(e) => {
            PointResultInner::Err(Error::cause("010", "run failure", e))
        }
    };
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

