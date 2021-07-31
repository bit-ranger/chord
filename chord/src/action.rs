use std::fmt::Display;

pub use async_trait::async_trait;

use crate::case::CaseId;
use crate::task::TaskId;
use crate::value::Map;
pub use crate::value::Value;
pub use crate::Error;
use std::time::Duration;

pub mod prelude {
    pub use crate::cause;
    pub use crate::err;
    pub use crate::value::*;

    pub use super::async_trait;
    pub use super::Action;
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

    fn args(&self, ctx: Option<Map>) -> Result<Value, Error>;

    fn timeout(&self) -> Duration;
}

pub trait CreateArg: Sync + Send {
    fn id(&self) -> &dyn CreateId;

    fn action(&self) -> &str;

    fn args_raw(&self) -> &Value;

    fn render_str(&self, text: &str) -> Result<String, Error>;

    /// shared in whole action
    fn is_shared(&self, text: &str) -> bool;
}

#[async_trait]
pub trait Action: Sync + Send {
    async fn run(&self, arg: &dyn RunArg) -> Result<Box<dyn Scope>, Error>;
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
