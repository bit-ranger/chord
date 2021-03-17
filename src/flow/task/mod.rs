use futures::future::join_all;

use crate::flow::case::model::CaseContextStruct;
use crate::flow::case;
use crate::flow::task::model::TaskContextStruct;
use crate::model::context::{AppContext, TaskResultStruct, TaskState, CaseResultInner};
use crate::model::context::TaskResultInner;
use itertools::Itertools;
use chrono::Utc;

pub mod model;

pub async fn run_task(app_context: &dyn AppContext, task_context: &TaskContextStruct) -> TaskResultInner {
    let start = Utc::now();

    let mut case_context_vec: Vec<CaseContextStruct> = task_context.create_case();

    let mut futures = case_context_vec.iter_mut().
        map(|case_context| run_case(app_context, case_context))
        .collect_vec();

    futures.reserve(0);
    let mut case_result_vec: Vec<(usize, CaseResultInner)> = Vec::new();
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

    let err_case = case_result_vec.iter()
        .filter(|(_, case)| case.is_err())
        .last();

    return match err_case {
        Some((_, ec)) => {
            let state = TaskState::CaseError(ec.as_ref().err().unwrap().clone());
            let result_struct = TaskResultStruct::new(case_result_vec, task_context.id(), start, Utc::now(), state);
            Ok(result_struct)
        }
        None => {
            let failure_case = case_result_vec.iter()
                .filter(|(_, case)| case.is_ok())
                .filter(|(_, case)| case.as_ref().unwrap().state().is_ok())
                .last();

            match failure_case {
                Some(_) => {
                    let result_struct = TaskResultStruct::new(case_result_vec, task_context.id(), start, Utc::now(), TaskState::CaseFailure);
                    Ok(result_struct)
                },
                None => {
                    let result_struct = TaskResultStruct::new(case_result_vec, task_context.id(), start, Utc::now(), TaskState::Ok);
                    Ok(result_struct)
                }
            }
       }
    }
}

async fn run_case(app_context: &dyn AppContext, case_context: &mut CaseContextStruct<'_, '_>) -> (usize, CaseResultInner){
    let case_result = case::run(app_context, case_context).await;
    return (case_context.id(), case_result);
}