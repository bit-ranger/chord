use core::result::Result::Ok;
use std::time::Duration;

use futures::future::join_all;
use tower::Service;
use tower::ServiceExt;

use crate::flow::case::model::CaseContextStruct;
use crate::flow::case::run_case;
use crate::flow::task::model::TaskContextStruct;
use crate::model::context::{AppContext, CaseResult};
use crate::model::context::TaskResult;
use crate::model::error::Error;

pub mod model;

pub async fn run_task(app_context: &dyn AppContext, task_context: &TaskContextStruct) -> TaskResult {
    let mut case_vec: Vec<CaseContextStruct> = task_context.create_case();

    // let (num, sec) = task_context.get_rate_limit();
    // let mut service = tower::ServiceBuilder::new()
    //     .rate_limit(num as u64, Duration::from_secs(sec as u64)) // 100 requests every 10 seconds
    //     .service_fn(run_case_wrap);


    // let mut futures = Vec::new();
    // let mut case_value_vec:Vec<CaseResult> = Vec::new();
    // for case in case_vec.iter_mut(){
    //     let case_value = match service.ready().await {
    //         Ok(r) => r.call((app_context, case)).await,
    //         Err(_) => Err((Error::new("tower", "rate limit error"), Vec::new()))
    //     };
    //     case_value_vec.push(case_value);
    // }

    // futures.reserve(0);
    // let mut case_value_vec = Vec::new();
    // let max_concurrency = task_context.max_concurrency();
    // loop {
    //     if futures.len() >  max_concurrency{
    //         let off = futures.split_off(futures.len() - max_concurrency);
    //         case_value_vec.extend(join_all(off).await);
    //     } else {
    //         case_value_vec.extend(join_all(futures).await);
    //         break;
    //     }
    // }


    let case_future_vec =
        case_vec
            .iter_mut()
            .map(|case| run_case(app_context, case));

    let case_value_vec = join_all(case_future_vec).await;

    let any_err = case_value_vec.iter()
        .any(|case| !case.is_ok());

    return if any_err {
        Err(
            Error::attach("000", "any case failure",
            case_value_vec))
    } else {
        Ok(case_value_vec)
    }
}

async fn run_case_wrap(ctx: (&dyn AppContext, &mut CaseContextStruct<'_,'_> )) -> CaseResult{
    let (app, case) = ctx;
    return run_case(app, case).await;
}

