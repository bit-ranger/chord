use std::fmt::Display;
use std::time::Duration;

pub use async_trait::async_trait;

use crate::case::CaseId;
use crate::task::TaskId;
use crate::value::Map;
use crate::value::Value;

pub type Error = Box<dyn std::error::Error + Sync + Send>;

pub mod prelude {
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
    pub use super::async_trait;
    pub use super::CreateArg;
    pub use super::Error;
    pub use super::Factory;
    pub use super::RunArg;
    pub use super::Scope;
}

pub trait Scope: Sync + Send {
    fn as_value(&self) -> &Value;
}

pub trait RunArg: Sync + Send {
    fn id(&self) -> &dyn RunId;

    fn context(&mut self) -> &mut Map;

    fn args(&self) -> Result<Value, Error>;
}

pub trait CreateArg: Sync + Send {
    fn id(&self) -> &dyn CreateId;

    fn action(&self) -> &str;

    fn args_raw(&self) -> &Value;

    fn render_str(&self, text: &str) -> Result<Value, Error>;

    /// shared in whole action
    fn is_static(&self, text: &str) -> bool;
}

#[async_trait]
pub trait Action: Sync + Send {
    async fn run(&self, arg: &mut dyn RunArg) -> Result<Box<dyn Scope>, Error>;

    async fn explain(&self, arg: &mut dyn RunArg) -> Result<Value, Error> {
        arg.args()
    }
}

#[async_trait]
pub trait Factory: Sync + Send {
    async fn create(&self, arg: &dyn CreateArg) -> Result<Box<dyn Action>, Error>;
}

impl Scope for Value {
    fn as_value(&self) -> &Value {
        &self
    }
}

pub trait CreateId: Sync + Send + Display {
    fn step(&self) -> &str;

    fn task_id(&self) -> &dyn TaskId;
}

pub trait RunId: Sync + Send + Display {
    fn step(&self) -> &str;

    fn case_id(&self) -> &dyn CaseId;
}
