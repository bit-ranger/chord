use async_std::task::spawn;
use core::result::Result;
use core::result::Result::Ok;
use std::thread;
use crate::case::run_case;
use crate::model::{TaskContext, CaseContext};
use futures::future::join_all;
use async_std::sync::Arc;

pub async fn run_task(task_context: Arc<TaskContext>) -> Result<(),()>{
    let tc_context_vec: Vec<Arc<CaseContext>> = task_context
        .create_case()
        .into_iter()
        .map(|tc_ctx| Arc::new(tc_ctx))
        .collect();

    join_all(tc_context_vec
        .iter()
        .map(|tc_context| run_case(tc_context.clone()))
        .map(|tc_future| spawn(tc_future))
    ).await;

    println!("run_task on thread {:?}", thread::current().id());
    return Ok(());
}
