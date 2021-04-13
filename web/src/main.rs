use chord_common::error::Error;

mod controller;

mod framework;
mod service;



#[async_std::main]
async fn main() -> Result<(), Error> {
    framework::init().await?;
    Ok(())
}


