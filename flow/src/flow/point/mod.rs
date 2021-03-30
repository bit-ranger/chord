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


pub async fn run(app_ctx: &dyn AppContext, arg: &PointArgStruct<'_, '_, '_, '_, '_>) -> PointAssessStruct
{
    trace!("point start {}", arg.id());
    let start = Utc::now();

    let future = app_ctx.get_point_runner().run(arg.kind(), arg);
    let timeout_value = timeout(arg.timeout(), future).await;
    let value = match timeout_value {
        Ok(v) => v,
        Err(_) => {
            warn!("point Err {}", arg.id());
            return PointAssessStruct::new(arg.id(), start, Utc::now(), PointState::Err(Error::new("002", "timeout")));
        }
    };

    return match value {
        PointValue::Ok(json) => {
            let assert_true = assert(arg, &json).await;
            return if assert_true {
                debug!("point Ok   {} - {} \n", arg.id(), json);
                PointAssessStruct::new(arg.id(), start, Utc::now(), PointState::Ok(json))
            } else {
                let txt = arg.render(arg.config().to_string().as_str()).unwrap();
                info!("point Fail {} - {} \n<<< {}", arg.id(), json, txt);
                PointAssessStruct::new(arg.id(), start, Utc::now(), PointState::Fail(json))
            }
        },
        PointValue::Err(e) => {
            let txt = arg.render(arg.config().to_string().as_str());
            warn!("point Err  {} - {} \n<<< {}", arg.id(), e, txt.unwrap());
            PointAssessStruct::new(arg.id(), start, Utc::now(), PointState::Err(e))
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

