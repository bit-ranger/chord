use crate::model::PointContext;
use crate::model::PointResult;
use serde_json::Value;

pub async fn run_point(context: &PointContext<'_,'_>) -> PointResult{
    let url = context.get_config_str(vec!["url"]).await.unwrap();

    let json :surf::Result<Value> = surf::get(&url)
        .header("Content-Type", "application/json")
        .recv_json()
        .await;

    match json {
        Ok(value) => {
            println!("{}", value);
            return Ok(value);
        },
        Err(e) => {
            println!("{}, {}, {}", url, "not a json", e);
            return Err(());
        }
    }

}