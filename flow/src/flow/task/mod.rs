use chrono::Utc;
use futures::future::join_all;
use itertools::Itertools;

use result::TaskResultStruct;

use crate::flow::case;
use crate::flow::case::arg::CaseArgStruct;
use crate::flow::task::arg::TaskArgStruct;
use crate::model::app::AppContext;
use common::task::{TaskState, TaskResult};
use common::case::CaseResult;
use common::err;
use common::value::{Json, Map};
use common::error::Error;
use log::info;

pub mod arg;
pub mod result;

pub async fn run_task(app_context: &dyn AppContext, task_context: &TaskArgStruct) -> TaskResult {
    let start = Utc::now();

    let case_ctx = pre_case(app_context, task_context).await?;
    info!("pre_case: {:?}", case_ctx);

    let mut data_case_arg_vec: Vec<CaseArgStruct> = task_context.data_case(&case_ctx);

    let mut futures = data_case_arg_vec.iter_mut().
        map(|case_context| run_case(app_context, case_context))
        .collect_vec();

    futures.reserve(0);
    let mut case_result_vec: Vec<(usize, CaseResult)> = Vec::new();
    let limit_concurrency = task_context.limit_concurrency();
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
            Ok(Box::new(result_struct))
        }
        None => {
            let failure_case = case_result_vec.iter()
                .filter(|(_, case)| case.is_ok())
                .filter(|(_, case)| !case.as_ref().unwrap().state().is_ok())
                .last();

            match failure_case {
                Some(_) => {
                    let result_struct = TaskResultStruct::new(case_result_vec, task_context.id(), start, Utc::now(), TaskState::CaseFailure);
                    Ok(Box::new(result_struct))
                },
                None => {
                    let result_struct = TaskResultStruct::new(case_result_vec, task_context.id(), start, Utc::now(), TaskState::Ok);
                    Ok(Box::new(result_struct))
                }
            }
       }
    }
}

async fn run_case(app_context: &dyn AppContext, case_arg: &mut CaseArgStruct<'_, '_,'_>) -> (usize, CaseResult){
    let case_result = case::run(app_context, case_arg).await;
    return (case_arg.id(), case_result);
}

async fn pre_case(app_context: &dyn AppContext, task_context: &TaskArgStruct) -> Result<Vec<(String, Json)>, Error>{
    let mut case_ctx = vec![];
    let mut pre_case = task_context.pre_case();
    match &mut pre_case {
        Some(pre_case) => {
            let (_, pre_result) = run_case(app_context, pre_case).await;
            if !pre_result.is_ok(){
                return err!("010", "pre run err");
            }
            let pre_assess = pre_result.unwrap();
            if !pre_assess.state().is_ok(){
                return err!("011", "pre run failure");
            }

            let mut pre_ctx = Map::new();
            for (pid, pr) in pre_assess.result(){
                match pr {
                    Ok(pv) => {
                        pre_ctx.insert(String::from(pid), pv.result().clone());
                    },
                    Err(_) => {
                        return err!("012", "pre point run failure");
                    }
                }
            }
            case_ctx.push((String::from("pre"), Json::Object(pre_ctx)));
        },
        None => {}
    }
    return Ok(case_ctx);
}