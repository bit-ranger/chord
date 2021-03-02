

use crate::model::CaseContext;
use std::thread;
use async_std::sync::Arc;
use crate::point::run_point;

pub async fn run_case(context: CaseContext) -> Result<(),()>{
    let point_vec = Arc::new(context).create_point();

    for point in point_vec {
        let _ = run_point(point).await;
    }

    // println!("run_case on thread {:?}", thread::current().id());
    return Ok(());
}