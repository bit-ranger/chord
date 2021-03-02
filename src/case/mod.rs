use crate::model::CaseContext;
use std::thread;

pub async fn run_case(context: CaseContext) -> Result<(),()>{
    println!("run_case {:?} on thread {:?}", context, thread::current().id());
    return Ok(());
}