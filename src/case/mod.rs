

use crate::model::{CaseContext, PointContext};
use crate::point::run_point;

pub async fn run_case<'t, 'c>(context: &'c mut CaseContext<'t>) -> Result<(),()>{
    let mut point_vec: Vec<PointContext> = context.create_point();

    for point in point_vec.iter_mut() {
        let _ = run_point(point).await;
        // match result {
        //     Ok(r) =>
        //     Err(_) => break
        // }
    }

    // println!("{:?}", point_vec);
    // println!("run_case on thread {:?}", thread::current().id());
    return Ok(());
}