use crate::model::CaseContext;
use std::thread;
use async_std::sync::Arc;

pub async fn run_case(context: CaseContext) -> Result<(),()>{
    let point_vec = Arc::new(context).create_point();
    println!("run_case {:?} on thread {:?}", point_vec, thread::current().id());
    return Ok(());
}