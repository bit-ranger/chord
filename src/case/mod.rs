

use crate::model::{CaseContext, PointContext};
use std::thread;
use async_std::sync::Arc;
use crate::point::run_point;

pub async fn run_case(context: Arc<CaseContext>) -> Result<(),()>{
    let point_vec: Vec<Arc<PointContext>> = context
        .create_point()
        .into_iter()
        .map(|point_ctx| Arc::new(point_ctx))
        .collect();

    for point in point_vec.iter() {
        let _ = run_point(point.clone()).await;
    }

    println!("{:?}", point_vec);
    // println!("run_case on thread {:?}", thread::current().id());
    return Ok(());
}