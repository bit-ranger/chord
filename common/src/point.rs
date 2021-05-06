pub use async_trait::async_trait;
use chrono::{DateTime, Utc};

use crate::error::Error;
use crate::value::Json;

pub type PointValue = std::result::Result<Json, Error>;

pub trait PointArg: Sync+Send {

    fn config(&self) -> &Json;

    fn render(&self, text: &str) -> Result<String, Error>;

    fn is_shared(&self, text: &str) -> bool;
}

#[async_trait]
pub trait PointRunner : Sync+Send{

    async fn run(&self, arg: &dyn PointArg) -> PointValue;
}

#[async_trait]
pub trait PointRunnerFactory : Sync+Send{

    async fn create_runner(&self, kind: &str, arg: &dyn PointArg) -> Result<Box<dyn PointRunner>, Error>;
}

#[derive(Debug, Clone)]
pub enum PointState {
    Ok(Json),
    Fail(Json),
    Err(Error)
}

pub trait PointAssess : Sync + Send{

    fn id(&self) -> &str;

    fn start(&self) -> DateTime<Utc>;

    fn end(&self) -> DateTime<Utc>;

    fn state(&self) -> &PointState;
}

unsafe impl Send for PointState
{
}

unsafe impl Sync for PointState
{
}

