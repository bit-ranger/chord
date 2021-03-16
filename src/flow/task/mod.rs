use futures::future::join_all;

use crate::flow::case::model::CaseContextStruct;
use crate::flow::case::run_case;
use crate::flow::task::model::TaskContextStruct;
use crate::model::context::{AppContext, TaskResultStruct, TaskError};
use crate::model::context::TaskResult;
use itertools::Itertools;
use chrono::Utc;

pub mod model;

pub async fn run_task(app_context: &dyn AppContext, task_context: &TaskContextStruct) -> TaskResult {
    let start = Utc::now();

    let mut case_vec: Vec<CaseContextStruct> = task_context.create_case();

    let mut futures = case_vec.iter_mut().
        map(|case| run_case(app_context, case))
        .collect_vec();

    futures.reserve(0);
    let mut case_result_vec = Vec::new();
    let limit_concurrency = task_context.get_limit_concurrency();
    loop {
        if futures.len() >  limit_concurrency{
            let off = futures.split_off(futures.len() - limit_concurrency);
            case_result_vec.extend(join_all(off).await);
        } else {
            case_result_vec.extend(join_all(futures).await);
            break;
        }
    }

    let any_err = case_result_vec.iter()
        .any(|case| !case.is_ok());

    let result_struct = TaskResultStruct::new(case_result_vec, task_context.id(), start, Utc::now());
    return if any_err {
        TaskResult::Err(TaskError::attach("010", "any case failure", result_struct))
    } else {
        TaskResult::Ok(result_struct)
    }
}