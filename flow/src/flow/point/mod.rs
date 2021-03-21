use chrono::Utc;

use common::error::Error;
use common::point::{PointValue, PointArg};
use common::value::Json;
use result::PointAssessStruct;

use crate::flow::point::arg::PointArgStruct;
use crate::model::app::AppContext;
use common::point::{PointState};
use log::{info, error};

pub mod arg;
pub mod result;


pub async fn run(app_context: &dyn AppContext, point_arg: &PointArgStruct<'_, '_, '_, '_, '_>) -> PointAssessStruct
{
    let start = Utc::now();
    let point_type = point_arg.meta_str(vec!["type"]).await;
    if point_type.is_none(){
        return PointAssessStruct::new(point_arg.id(), start, Utc::now(), PointState::Err(Error::new("001", "missing type")));
    }
    let point_type = point_type.unwrap();
    let point_runner = app_context.get_point_runner();
    let point_value = point_runner.run(point_type.as_str(), point_arg).await;

    return match point_value {
        PointValue::Ok(json) => {
            let assert_true = assert(point_arg, &json).await;
            return if assert_true {
                PointAssessStruct::new(point_arg.id(), start, Utc::now(), PointState::Ok(json))
            } else {
                let txt = point_arg.render(point_arg.config().to_string().as_str()).unwrap();
                info!("PointValue: {} - Fail - {} \n>>> {}", point_type, json, txt);
                PointAssessStruct::new(point_arg.id(), start, Utc::now(), PointState::Fail(json))
            }
        },
        PointValue::Err(e) => {
            let txt = point_arg.render(point_arg.config().to_string().as_str());
            error!("PointValue: {} - Err  - {} \n>>> {}", point_type, e, txt.unwrap());
            PointAssessStruct::new(point_arg.id(), start, Utc::now(), PointState::Err(e))
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

