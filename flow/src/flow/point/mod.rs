use chrono::Utc;

use common::error::Error;
use common::point::PointValue;
use common::value::Json;
use result::PointAssessStruct;

use crate::flow::point::arg::PointArgStruct;
use crate::model::app::AppContext;
use common::point::{PointResult, PointState};

pub mod arg;
pub mod result;


pub async fn run(app_context: &dyn AppContext, point_arg: &PointArgStruct<'_, '_, '_, '_, '_>) -> PointResult
{
    let start = Utc::now();
    let point_type = point_arg.meta_str(vec!["type"]).await;
    if point_type.is_none(){
        return Err(Error::new("001", "missing type"));
    }
    let point_type = point_type.unwrap();
    let point_runner = app_context.get_point_runner();
    let result = point_runner.run(point_type.as_str(), point_arg).await;
    let end = Utc::now();

    return match result{
        PointValue::Ok(json) => {
            let assert_true = assert(point_arg, &json).await;
            return if assert_true {
                let result_struct = PointAssessStruct::new(json, point_arg.id(), start, end, PointState::Ok);
                Ok(Box::new(result_struct))
            } else {
                let result_struct = PointAssessStruct::new(json, point_arg.id(), start, end, PointState::Failure);
                Ok(Box::new(result_struct))
            }
        },
        PointValue::Err(e) => {
            let result_struct = PointAssessStruct::new(Json::Null, point_arg.id(), start, end, PointState::Error(e));
            Ok(Box::new(result_struct))
        }
    };
}

async fn assert(context: &PointArgStruct<'_, '_, '_, '_, '_>, result: &Json) -> bool{
    let assert_condition = context.meta_str(vec!["assert"]).await;
    return match assert_condition{
        Some(con) =>  {
            context.assert(con.as_str(), &result).await
        },
        None => true
    }
}

