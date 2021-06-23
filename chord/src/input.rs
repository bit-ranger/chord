pub use async_trait::async_trait;

use crate::error::Error;
use crate::value::Value;

#[async_trait]
pub trait CaseLoad: Sync + Send {
    async fn load(&mut self, size: usize) -> Result<Vec<(String, Value)>, Error>;

    async fn reset(&mut self) -> Result<(), Error>;
}
