use crate::model::{Error, Json, PointResult, PointContext};

pub async fn run_point(context: &dyn PointContext) -> PointResult{
    let url = context.get_config_str(vec!["url"]).unwrap();

    println!("url {}", url);

    let json :surf::Result<Json> = surf::get(url.as_str())
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
            return Err(Error::new("000", "response is not a json"));
        }
    }

}