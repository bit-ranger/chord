use tide::prelude::*;
use chord_common::error::Error;
use validator::{Validate};



#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct Job {
    #[validate(length(min = 1, max = 10))]
    name: String
}

pub async fn exec(job: Job) -> Result<Job, Error> {
    return Ok(job);
}