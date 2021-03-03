

use crate::model::{CaseContext, PointContext, SharedCaseContext, SharedPointContext};
use std::thread;
use async_std::sync::{Arc, RwLock};
use crate::point::run_point;
use futures::TryFutureExt;
use std::ops::{Deref, DerefMut};

pub async fn run_case(context: SharedCaseContext) -> Result<(),()>{
    let point_vec: Vec<SharedPointContext> = CaseContext::create_point(context).await
        .into_iter()
        .map(|point_ctx|point_ctx.share())
        .collect();

    for mut point in point_vec.into_iter() {
        let result = run_point(point).await;
        // match result {
        //     Ok(r) =>
        //     Err(_) => break
        // }
    }

    // println!("{:?}", point_vec);
    // println!("run_case on thread {:?}", thread::current().id());
    return Ok(());
}