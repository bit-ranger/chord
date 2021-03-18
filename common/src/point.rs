use std::future::Future;
use std::pin::Pin;


use crate::error::Error;
use crate::value::Json;
use chrono::{DateTime, Utc};

pub type PointValue = std::result::Result<Json, Error>;

pub trait PointArg {

    fn get_config_rendered(&self, path: Vec<&str>) -> Option<String>;

    fn get_config(&self) -> &Json;

    fn render(&self, text: &str) -> Result<String,Error>;
}

pub trait PointRunner{

    fn run<'a>(&self, point_type: &'a str, point_arg: &'a dyn PointArg) -> Pin<Box<dyn Future<Output=PointValue>+ 'a>>;
}

pub type PointResult = std::result::Result<Box<dyn PointAssess>, Error>;

pub trait PointAssess {

    fn id(&self) -> &str;

    fn start(&self) -> DateTime<Utc>;

    fn end(&self) -> DateTime<Utc>;

    fn result(&self) -> &Json;
}
