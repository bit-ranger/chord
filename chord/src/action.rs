use std::fmt::Display;

pub use async_trait::async_trait;

use crate::case::CaseId;
use crate::task::TaskId;
use crate::value::Map;
use crate::value::Value;
use std::error::Error;
use std::time::Duration;

pub mod prelude {
    pub use super::async_trait;
    pub use super::Action;
    pub use super::CreateArg;
    pub use super::Factory;
    pub use super::RunArg;
    pub use super::Scope;
    pub use crate::value::*;
    pub use std::error::Error;
}

pub trait Scope: Sync + Send {
    fn as_value(&self) -> &Value;
}

pub trait RunArg: Sync + Send {
    fn id(&self) -> &dyn RunId;

    fn context(&self) -> &Map;

    fn timeout(&self) -> Duration;

    fn args(&self) -> Result<Value, Box<dyn Error>>;

    fn args_with(&self, context: &Map) -> Result<Value, Box<dyn Error>>;
}

pub trait CreateArg: Sync + Send {
    fn id(&self) -> &dyn CreateId;

    fn action(&self) -> &str;

    fn args_raw(&self) -> &Value;

    fn render_str(&self, text: &str) -> Result<Value, Box<dyn Error>>;

    /// shared in whole action
    fn is_static(&self, text: &str) -> bool;
}

#[async_trait]
pub trait Action: Sync + Send {
    async fn run(&self, arg: &dyn RunArg) -> Result<Box<dyn Scope>, Box<dyn Error>>;

    async fn explain(&self, arg: &dyn RunArg) -> Result<Value, Box<dyn Error>> {
        arg.args()
    }
}

#[async_trait]
pub trait Factory: Sync + Send {
    async fn create(&self, arg: &dyn CreateArg) -> Result<Box<dyn Action>, Box<dyn Error>>;
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
