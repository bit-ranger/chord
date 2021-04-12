mod controller;
use controller::job;

#[async_std::main]
async fn main() -> tide::Result<()> {
    let mut app = tide::new();

    app.at("/job/exec").post(job::job_exec);

    app.listen("127.0.0.1:8080").await?;

    Ok(())
}


