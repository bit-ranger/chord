use chrono::Utc;

use common::error::Error;
use common::point::{PointValue, PointArg};
use common::value::Json;
use res::PointAssessStruct;

use crate::flow::point::arg::PointArgStruct;
use crate::model::app::AppContext;
use common::point::{PointState};
use log::{debug, info, error};

pub mod arg;
pub mod res;


pub async fn run(app_context: &dyn AppContext, point_arg: &PointArgStruct<'_, '_, '_, '_, '_>) -> PointAssessStruct
{
    let start = Utc::now();
    let pt_type = point_arg.meta_str(vec!["type"]).await;
    if pt_type.is_none(){
        return PointAssessStruct::new(point_arg.id(), start, Utc::now(), PointState::Err(Error::new("001", "missing type")));
    }
    let pt_type = pt_type.unwrap();
    let pt_runner = app_context.get_pt_runner();
    let pt_value = pt_runner.run(pt_type.as_str(), point_arg).await;

    return match pt_value {
        PointValue::Ok(json) => {
            let assert_true = assert(point_arg, &json).await;
            return if assert_true {
                debug!("PointValue: {} - Ok   - {} \n", point_arg.id(), json);
                PointAssessStruct::new(point_arg.id(), start, Utc::now(), PointState::Ok(json))
            } else {
                let txt = point_arg.render(point_arg.config().to_string().as_str()).unwrap();
                info!("PointValue: {} - Fail - {} \n<<< {}", point_arg.id(), json, txt);
                PointAssessStruct::new(point_arg.id(), start, Utc::now(), PointState::Fail(json))
            }
        },
        PointValue::Err(e) => {
            let txt = point_arg.render(point_arg.config().to_string().as_str());
            error!("PointValue: {} - Err  - {} \n<<< {}", point_arg.id(), e, txt.unwrap());
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

