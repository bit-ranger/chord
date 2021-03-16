use core::result::Result::Ok;
use futures::future::join_all;

use crate::flow::case::model::CaseContextStruct;
use crate::flow::case::run_case;
use crate::flow::task::model::TaskContextStruct;
use crate::model::context::{AppContext};
use crate::model::context::TaskResult;
use crate::model::error::Error;
use itertools::Itertools;

pub mod model;

pub async fn run_task(app_context: &dyn AppContext, task_context: &TaskContextStruct) -> TaskResult {
    let mut case_vec: Vec<CaseContextStruct> = task_context.create_case();

    let mut futures = case_vec.iter_mut().
        map(|case| run_case(app_context, case))
        .collect_vec();

    futures.reserve(0);
    let mut case_value_vec = Vec::new();
    let limit_concurrency = task_context.get_limit_concurrency();
    loop {
        if futures.len() >  limit_concurrency{
            let off = futures.split_off(futures.len() - limit_concurrency);
            case_value_vec.extend(join_all(off).await);
        } else {
            case_value_vec.extend(join_all(futures).await);
            break;
        }
    }

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