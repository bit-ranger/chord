use crate::model::PointContext;
use std::thread;
use async_std::sync::Arc;

pub async fn run_point(context: PointContext) -> Result<(),()>{
    let url = context.get_config()["config"]["url"].as_str().unwrap();
    let url = context.render(url, Option::None);

    let assert_condition = context.get_config()["assert"].as_str().unwrap();
    let assert_result = context.assert(assert_condition, Option::None);
    println!("run_point {} {} on thread {:?}", url, assert_result, thread::current().id());
    return Ok(());
}