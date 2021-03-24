use chrono::Utc;

use common::error::Error;
use common::point::{PointValue, PointArg};
use common::value::Json;
use res::PointAssessStruct;

use crate::flow::point::arg::PointArgStruct;
use crate::model::app::AppContext;
use common::point::{PointState};
use async_std::future::timeout;
use log::{trace, debug, info, warn};

pub mod arg;
pub mod res;


pub async fn run(app_context: &dyn AppContext, point_arg: &PointArgStruct<'_, '_, '_, '_, '_>) -> PointAssessStruct
{
    trace!("point start {}", point_arg.id());
    let start = Utc::now();
    let pt_type = point_arg.meta_str(vec!["type"]).await;
    if pt_type.is_none(){
        warn!("point Err {}", point_arg.id());
        return PointAssessStruct::new(point_arg.id(), start, Utc::now(), PointState::Err(Error::new("001", "missing type")));
    }
    let pt_type = pt_type.unwrap();
    let pt_runner = app_context.get_pt_runner();
    let pt_value_future = pt_runner.run(pt_type.as_str(), point_arg);
    let timeout_result = timeout(point_arg.timeout(), pt_value_future).await;
    let pt_value = match timeout_result {
        Ok(pt_value) => pt_value,
        Err(_) => {
            warn!("point Err {}", point_arg.id());
            return PointAssessStruct::new(point_arg.id(), start, Utc::now(), PointState::Err(Error::new("002", "timeout")));
        }
    };

    return match pt_value {
        PointValue::Ok(json) => {
            let assert_true = assert(point_arg, &json).await;
            return if assert_true {
                debug!("point Ok   {} - {} \n", point_arg.id(), json);
                PointAssessStruct::new(point_arg.id(), start, Utc::now(), PointState::Ok(json))
            } else {
                let txt = point_arg.render(point_arg.config().to_string().as_str()).unwrap();
                info!("point Fail {} - {} \n<<< {}", point_arg.id(), json, txt);
                PointAssessStruct::new(point_arg.id(), start, Utc::now(), PointState::Fail(json))
            }
        },
        PointValue::Err(e) => {
            let txt = point_arg.render(point_arg.config().to_string().as_str());
            warn!("point Err  {} - {} \n<<< {}", point_arg.id(), e, txt.unwrap());
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

