use std::fmt::Display;

pub use async_trait::async_trait;

use crate::case::CaseId;
use crate::value::Map;
use crate::value::Value;

pub type Error = Box<dyn std::error::Error + Sync + Send>;

pub mod prelude {
    pub use crate::value::from_reader;
    pub use crate::value::from_slice;
    pub use crate::value::from_str;
    pub use crate::value::from_value;
    pub use crate::value::json;
    pub use crate::value::to_string;
    pub use crate::value::to_string_pretty;
    pub use crate::value::Deserialize;
    pub use crate::value::Map;
    pub use crate::value::Number;
    pub use crate::value::Serialize;
    pub use crate::value::Value;

    pub use super::async_trait;
    pub use super::Action;
    pub use super::Arg;
    pub use super::Error;
    pub use super::Factory;
    pub use super::Scope;
}

pub trait Id: Sync + Send + Display {
    fn step(&self) -> &str;

    fn case_id(&self) -> &dyn CaseId;
}

pub trait Context: Sync + Send {
    fn data(&self) -> &Map;

    fn data_mut(&mut self) -> &mut Map;
}

pub trait Scope: Sync + Send {
    fn as_value(&self) -> &Value;
}

impl Scope for Value {
    fn as_value(&self) -> &Value {
        &self
    }
}

pub trait Arg: Sync + Send {
    fn id(&self) -> &dyn Id;

    fn args(&self) -> Result<Value, Error>;

    fn args_raw(&self) -> &Value;

    fn context(&self) -> &dyn Context;

    fn render(&self, context: &dyn Context, raw: &Value) -> Result<Value, Error>;

    fn factory(&self, action: &str) -> Option<&dyn Factory>;

    fn is_static(&self, raw: &Value) -> bool;
}

#[async_trait]
pub trait Action: Sync + Send {
    async fn run(&self, arg: &dyn Arg) -> Result<Box<dyn Scope>, Error>;

    async fn explain(&self, arg: &dyn Arg) -> Result<Value, Error> {
        arg.args()
    }
}

#[async_trait]
pub trait Factory: Sync + Send {
    async fn create(&self, arg: &dyn Arg) -> Result<Box<dyn Action>, Error>;
}
