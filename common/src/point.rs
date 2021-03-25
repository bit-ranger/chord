use std::future::Future;
use std::pin::Pin;


use crate::error::Error;
use crate::value::Json;
use chrono::{DateTime, Utc};

pub type PointValue = std::result::Result<Json, Error>;

pub trait PointArg: Sync+Send {

    fn config_rendered(&self, path: Vec<&str>) -> Option<String>;

    fn config(&self) -> &Json;

    fn render(&self, text: &str) -> Result<String,Error>;
}

pub trait PointRunner: Sync+Send{

    fn run<'a>(&self, pt_type: &'a str, pt_arg: &'a dyn PointArg) -> Pin<Box<dyn Future<Output=PointValue>+ Send + 'a>>;
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

