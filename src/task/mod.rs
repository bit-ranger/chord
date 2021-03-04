// use async_std::task::spawn;
use core::result::Result;
use core::result::Result::Ok;
use crate::case::run_case;
use crate::model::{TaskContext, CaseContext};
use futures::future::join_all;

pub async fn run_task(task_context: &mut TaskContext) -> Result<(),()>{
    let mut case_vec: Vec<CaseContext> = task_context.create_case();

    let mut futures = Vec::new();
    for case in case_vec.iter_mut(){
        futures.push(
        // spawn(async move {
                run_case(case)
        // })
        );
    }



    join_all(futures).await;

    return Ok(());
}
