use chrono::Utc;
use futures::future::join_all;
use itertools::Itertools;
use log::info;

use common::err;
use common::error::Error;
use common::task::{TaskAssess, TaskState};
use common::value::{Json, Map};
use result::TaskAssessStruct;

use crate::flow::case;
use crate::flow::case::arg::CaseArgStruct;
use crate::flow::task::arg::TaskArgStruct;
use crate::model::app::AppContext;
use common::case::{CaseAssess, CaseState};

pub mod arg;
pub mod result;

pub async fn run_task(app_context: &dyn AppContext, task_context: &TaskArgStruct) -> TaskAssessStruct {
    let start = Utc::now();

    let case_ctx = pre_case(app_context, task_context).await?;

    let mut data_case_arg_vec: Vec<CaseArgStruct> = task_context.data_case(&case_ctx);

    let mut futures = data_case_arg_vec.iter_mut().
        map(|case_context| case::run(app_context, case_arg))
        .collect_vec();

    futures.reserve(0);
    let mut case_assess_vec: Vec<dyn CaseAssess> = Vec::new();
    let limit_concurrency = task_context.limit_concurrency();
    loop {
        if futures.len() >  limit_concurrency{
            let off = futures.split_off(futures.len() - limit_concurrency);
            case_assess_vec.extend(join_all(off).await);
        } else {
            case_assess_vec.extend(join_all(futures).await);
            break;
        }
    }

    let err_case = case_assess_vec.iter()
        .filter(|(_, case)| case.is_err())
        .last();

    return match err_case {
        Some((_, ec)) => {
            let state = TaskState::CaseError(ec.as_ref().err().unwrap().clone());
            let result_struct = TaskAssessStruct::new(case_assess_vec, task_context.id(), start, Utc::now(), state);
            Ok(Box::new(result_struct))
        }
        None => {
            let failure_case = case_assess_vec.iter()
                .filter(|(_, case)| case.is_ok())
                .filter(|(_, case)| !case.as_ref().unwrap().state().is_ok())
                .last();

            match failure_case {
                Some(_) => {
                    let result_struct = TaskAssessStruct::new(case_assess_vec, task_context.id(), start, Utc::now(), TaskState::CaseFailure);
                    Ok(Box::new(result_struct))
                },
                None => {
                    let result_struct = TaskAssessStruct::new(case_assess_vec, task_context.id(), start, Utc::now(), TaskState::Ok);
                    Ok(Box::new(result_struct))
                }
            }
       }
    }
}


async fn pre_case(app_context: &dyn AppContext, task_context: &TaskArgStruct) -> Result<Vec<(String, Json)>, Error>{
    let mut case_ctx = vec![];
    let mut pre_case = task_context.pre_case();
    match &mut pre_case {
        Some(pre_case) => {
            let pre_assess = case::run(app_context, pre_case).await;

            match pre_assess.state(){
                CaseState::Ok(pa_vec) => {
                    let mut pre_ctx = Map::new();
                    for pa in pa_vec {
                        match pa {
                            PointState::Ok(pv) => {
                                pre_ctx.insert(String::from(pid), pv.result().clone());
                            },
                            _ => {
                                return err!("012", "pre point run failure");
                            }
                        }
                    }
                    let pre = Json::Object(pre_ctx);
                    info!("pre_case: {:?}", pre);
                    case_ctx.push((String::from("pre"), pre));
                }
                _ => return err!("011", "pre fail"),
            }



        },
        None => {}
    }
    return Ok(case_ctx);
}