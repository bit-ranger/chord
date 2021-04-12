use tide::Request;
use tide::prelude::*;


#[derive(Debug, Deserialize)]
struct Job {
    name: String
}

pub async fn job_exec(mut req: Request<()>) -> tide::Result {
    let Job { name} = req.body_json().await?;
    Ok(format!("Hello, I've put in an job for {}", name).into())
}