// use async_std::task::spawn;
use core::result::Result::Ok;

use futures::future::join_all;

use crate::flow::case::model::CaseContextStruct;
use crate::flow::case::run_case;
use crate::flow::task::model::TaskContextStruct;
use crate::model::context::AppContext;
use crate::model::error::Error;
use crate::model::context::TaskResult;

pub mod model;

pub async fn run_task(app_context: &dyn AppContext, task_context: &TaskContextStruct) -> TaskResult {
    let mut case_vec: Vec<CaseContextStruct> = task_context.create_case();

    let mut futures = Vec::new();
    for case in case_vec.iter_mut(){
        futures.push(
        // spawn(async move {
                run_case(app_context,case)
        // })
        );
    }

    let case_value_vec = join_all(futures).await;

    let any_err = case_value_vec.iter()
        .any(|case| !case.is_ok());

    return if any_err {
        Err(Error::new("000", "any case failure"))
    } else {
        Ok(case_value_vec)
    }
}


