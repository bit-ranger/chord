use crate::error::Error;
use crate::value::Json;
pub use async_trait::async_trait;

#[async_trait]
pub trait DataLoad: Sync + Send {
    async fn load(&mut self, size: usize) -> Result<Vec<Json>, Error>;
}
