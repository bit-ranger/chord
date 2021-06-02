pub use async_trait::async_trait;
use chrono::{DateTime, Utc};

use crate::error::Error;
use crate::value::Json;
use crate::case::CaseId;
use std::fmt::Display;

pub type PointValue = std::result::Result<Json, Error>;


pub trait PointId: Sync + Send + Display{
    fn point_id(&self) -> &str;

    fn case_id(&self) -> &dyn CaseId;
}


pub trait RunArg: Sync + Send {

    fn id(&self) -> &dyn PointId;

    fn config(&self) -> &Json;

    fn render(&self, text: &str) -> Result<String, Error>;
}


pub trait CreateArg: Sync + Send {

    fn id(&self) -> &dyn PointId;

    fn kind(&self) -> &str;

    fn config(&self) -> &Json;

    fn render(&self, text: &str) -> Result<String, Error>;

    fn is_task_shared(&self, text: &str) -> bool;
}


#[derive(Debug, Clone)]
pub enum PointState {
    Ok(Json),
    Fail(Json),
    Err(Error),
}
unsafe impl Send for PointState {}
unsafe impl Sync for PointState {}


pub trait PointAssess: Sync + Send {

    fn id(&self) -> &dyn PointId;

    fn start(&self) -> DateTime<Utc>;

    fn end(&self) -> DateTime<Utc>;

    fn state(&self) -> &PointState;
}


#[async_trait]
pub trait PointRunner: Sync + Send {
    async fn run(&self, arg: &dyn RunArg) -> PointValue;
}

#[async_trait]
pub trait PointRunnerFactory: Sync + Send {
    async fn create(&self, arg: &dyn CreateArg) -> Result<Box<dyn PointRunner>, Error>;
}