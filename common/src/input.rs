use crate::error::Error;
use crate::value::Json;
pub use async_trait::async_trait;

#[async_trait]
pub trait CaseLoad: Sync + Send {

    async fn load(&mut self, size: usize) -> Result<Vec<Json>, Error>;

    async fn reset(&mut self) -> Result<(), Error>;
}
