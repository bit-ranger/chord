pub mod prelude {
    pub use super::async_trait;
    pub use super::Action;
    pub use super::ActionValue;
    pub use super::CreateArg;
    pub use super::Error;
    pub use super::Factory;
    pub use super::RunArg;
    pub use super::Value;
}

pub use async_trait::async_trait;

pub use crate::value::Value;
pub use crate::Error;

pub type ActionValue = std::result::Result<Value, Error>;

pub trait RunArg: Sync + Send {
    fn id(&self) -> &str;

    fn args(&self) -> &Value;

    fn render_str(&self, text: &str) -> Result<String, Error>;

    fn render_value(&self, text: &Value) -> Result<Value, Error>;
}

pub trait CreateArg: Sync + Send {
    fn id(&self) -> &str;

    fn action(&self) -> &str;

    fn args(&self) -> &Value;

    fn render_str(&self, text: &str) -> Result<String, Error>;

    /// shared in whole action
    fn is_shared(&self, text: &str) -> bool;
}

#[async_trait]
pub trait Action: Sync + Send {
    async fn run(&self, arg: &dyn RunArg) -> ActionValue;
}

#[async_trait]
pub trait Factory: Sync + Send {
    async fn create(&self, arg: &dyn CreateArg) -> Result<Box<dyn Action>, Error>;
}
