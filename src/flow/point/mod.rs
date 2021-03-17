use chrono::Utc;

use result::{PointAssessStruct};

use crate::flow::point::arg::PointArgStruct;
use crate::model::point::{PointValue, PointResult};
use crate::model::error::Error;
use crate::model::value::Json;
use crate::point::run_point_type;

pub mod arg;
pub mod result;


pub async fn run(context: &PointArgStruct<'_, '_, '_, '_, '_>) -> PointResult
{
    let start = Utc::now();
    let point_type = context.get_meta_str(vec!["type"]).await;
    if point_type.is_none(){
        return Err(Error::new("001", "missing type"));
    }
    let point_type = point_type.unwrap();

    let result = run_point_type(point_type.as_str(), context).await;
    let end = Utc::now();

    return match result{
        PointValue::Ok(json) => {
            let result_struct = PointAssessStruct::new(json, context.get_id(), start, end);
            Ok(Box::new(result_struct))
        },
        PointValue::Err(e) => {
            Err(Error::cause("010", "run failure", e))
        }
    };
}

pub async fn assert(context: &PointArgStruct<'_, '_, '_, '_, '_>, result: &Json) -> bool{
    let assert_condition = context.get_meta_str(vec!["assert"]).await;
    return match assert_condition{
        Some(con) =>  {
            context.assert(con.as_str(), &result).await
        },
        None => true
    }
}

