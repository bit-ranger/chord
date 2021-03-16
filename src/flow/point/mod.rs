use crate::flow::point::model::PointContextStruct;
use crate::model::error::Error;
use crate::model::context::{PointResult, PointResultStruct, PointResultInner, PointErrorInner};
use crate::point::run_point_type;
use crate::model::value::Json;
use chrono::Utc;

pub mod model;




pub async fn run_point(context: &PointContextStruct<'_, '_, '_, '_, '_>) -> PointResultInner
{
    let start = Utc::now();
    let point_type = context.get_meta_str(vec!["type"]).await;
    if point_type.is_none(){
        let end = Utc::now();
        let result_struct = PointResultStruct::new(Json::Null, context.get_id(), start, end);
        return PointResultInner::Err(Error::attach("001", "missing type", result_struct));
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
            let result_struct = PointResultStruct::new(Json::Null, context.get_id(), start, end);
            PointResultInner::Err(PointErrorInner::attach("010",
                                                format!("run failure cause: {}", e).as_str(),
                                                result_struct))
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

