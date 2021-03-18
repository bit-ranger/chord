use chrono::{DateTime, Utc};

use common::error::Error;
use common::value::Json;

pub type PointResult = std::result::Result<Box<dyn PointAssess>, Error>;

pub trait PointAssess {

    fn id(&self) -> &str;

    fn start(&self) -> DateTime<Utc>;

    fn end(&self) -> DateTime<Utc>;

    fn result(&self) -> &Json;
}