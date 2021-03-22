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


pub async fn run(app_context: &dyn AppContext, pt_arg: &PointArgStruct<'_, '_, '_, '_, '_>) -> PointAssessStruct
{
    let start = Utc::now();
    let pt_type = pt_arg.meta_str(vec!["type"]).await;
    if pt_type.is_none(){
        return PointAssessStruct::new(pt_arg.id(), start, Utc::now(), PointState::Err(Error::new("001", "missing type")));
    }
    let pt_type = pt_type.unwrap();
    let pt_runner = app_context.get_pt_runner();
    let pt_value = pt_runner.run(pt_type.as_str(), pt_arg).await;

    return match pt_value {
        PointValue::Ok(json) => {
            let assert_true = assert(pt_arg, &json).await;
            return if assert_true {
                PointAssessStruct::new(pt_arg.id(), start, Utc::now(), PointState::Ok(json))
            } else {
                let txt = pt_arg.render(pt_arg.config().to_string().as_str()).unwrap();
                info!("PointValue: {} - Fail - {} \n>>> {}", pt_type, json, txt);
                PointAssessStruct::new(pt_arg.id(), start, Utc::now(), PointState::Fail(json))
            }
        },
        PointValue::Err(e) => {
            let txt = pt_arg.render(pt_arg.config().to_string().as_str());
            error!("PointValue: {} - Err  - {} \n>>> {}", pt_type, e, txt.unwrap());
            PointAssessStruct::new(pt_arg.id(), start, Utc::now(), PointState::Err(e))
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

