

use crate::model::{CaseContext, PointContext};
use std::thread;
use async_std::sync::{Arc, RwLock};
use crate::point::run_point;
use futures::TryFutureExt;
use std::ops::{Deref, DerefMut};

pub async fn run_case(context: Arc<CaseContext>) -> Result<(),()>{
    let point_vec: Vec<Arc<RwLock<PointContext>>> = context
        .create_point()
        .into_iter()
        .map(|point_ctx| Arc::new(RwLock::new(point_ctx)))
        .collect();

    for mut point in point_vec.into_iter() {
        let result = run_point(point.write().await.deref_mut()).await;
        // match result {
        //     Ok(r) =>
        //     Err(_) => break
        // }
    }

    // println!("{:?}", point_vec);
    // println!("run_case on thread {:?}", thread::current().id());
    return Ok(());
}