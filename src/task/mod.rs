// use async_std::task::spawn;
use core::result::Result;
use core::result::Result::Ok;
use crate::case::run_case;
use crate::model::{TaskContext, CaseContext, TaskResult};
use futures::future::join_all;

pub async fn run_task(task_context: &TaskContext) -> TaskResult {
    let mut case_vec: Vec<CaseContext> = task_context.create_case();

    let mut futures = Vec::new();
    for case in case_vec.iter_mut(){
        futures.push(
        // spawn(async move {
                run_case(case)
        // })
        );
    }

    let case_value_vec = join_all(futures).await;

    let any_err = case_value_vec.iter()
        .any(|case| !case.is_ok());

    return if any_err {
        Err(())
    } else {
        Ok(case_value_vec)
    }
}
