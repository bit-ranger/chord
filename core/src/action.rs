use std::fmt::Display;

pub use async_trait::async_trait;
pub use chrono::{DateTime, Utc};

use crate::case::CaseId;
use crate::value::Map;
use crate::value::Value;

pub type Error = Box<dyn std::error::Error + Sync + Send>;

pub mod prelude {
    pub use chrono::{DateTime, Utc};

    pub use crate::value::Deserialize;
    pub use crate::value::from_reader;
    pub use crate::value::from_slice;
    pub use crate::value::from_str;
    pub use crate::value::from_value;
    pub use crate::value::json;
    pub use crate::value::Map;
    pub use crate::value::Number;
    pub use crate::value::Serialize;
    pub use crate::value::to_string;
    pub use crate::value::to_string_pretty;
    pub use crate::value::Value;

    pub use super::Action;
    pub use super::Arg;
    pub use super::Asset;
    pub use super::async_trait;
    pub use super::Chord;
    pub use super::Context;
    pub use super::Creator;
    pub use super::Data;
    pub use super::Error;
    pub use super::Id;
}

pub trait Id: Sync + Send + Display {
    fn step(&self) -> &str;

    fn case_id(&self) -> &dyn CaseId;

    fn clone(&self) -> Box<dyn Id>;
}

pub trait Context: Sync + Send {
    fn data(&self) -> &Map;

    fn data_mut(&mut self) -> &mut Map;

    fn clone(&self) -> Box<dyn Context>;
}


pub trait Data: Sync + Send {
    fn to_value(&self) -> Value;
}

impl Data for Value {
    fn to_value(&self) -> Value {
        self.clone()
    }
}


pub trait Frame: Data {

    fn id(&self) -> &str;

    fn start(&self) -> DateTime<Utc>;

    fn end(&self) -> DateTime<Utc>;
}


pub enum Asset {
    Value(Value),
    Data(Box<dyn Data>),
    Frames(Vec<Box<dyn Frame>>)
}

impl Asset {
    pub fn to_value(&self) -> Value {
        match self {
            Asset::Value(v) => {
                v.clone()
            }
            Asset::Data(d) => {
                d.to_value()
            }
            Asset::Frames(fs) => {
                Value::Array(fs.iter().map(|f| f.to_value()).collect())
            }
        }
    }

}

pub trait Chord: Sync + Send {
    fn creator(&self, action: &str) -> Option<&dyn Creator>;

    fn render(&self, context: &dyn Context, raw: &Value) -> Result<Value, Error>;

    fn clone(&self) -> Box<dyn Chord>;
}

pub trait Arg: Sync + Send {
    fn id(&self) -> &dyn Id;

    fn args(&self) -> Result<Value, Error>;

    fn args_raw(&self) -> &Value;

    fn args_init(&self) -> Option<&Value>;

    fn context(&self) -> &dyn Context;

    fn context_mut(&mut self) -> &mut dyn Context;
}

#[async_trait]
pub trait Action: Sync + Send {
    async fn execute(&self, chord: &dyn Chord, arg: &mut dyn Arg) -> Result<Asset, Error>;

    async fn explain(&self, _chord: &dyn Chord, arg: &dyn Arg) -> Result<Value, Error> {
        arg.args()
    }
}

#[async_trait]
pub trait Creator: Sync + Send {
    async fn create(&self, chord: &dyn Chord, arg: &dyn Arg) -> Result<Box<dyn Action>, Error>;
}
