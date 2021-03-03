use async_std::task::spawn;
use core::result::Result;
use core::result::Result::Ok;
use std::thread;
use crate::case::run_case;
use crate::model::{TaskContext, CaseContext, SharedCaseContext};
use futures::future::join_all;
use async_std::sync::Arc;

pub async fn run_task(task_context: Arc<TaskContext>) -> Result<(),()>{
    let share = task_context.share();
    let tc_context_vec: Vec<SharedCaseContext> = TaskContext::create_case(share).await
        .into_iter()
        .map(|tc_ctx| tc_ctx.share())
        .collect();

    join_all(tc_context_vec
        .iter()
        .map(|tc_context| run_case(tc_context.clone()))
        .map(|tc_future| spawn(tc_future))
    ).await;

    println!("run_task on thread {:?}", thread::current().id());
    return Ok(());
}
