use std::path::Path;

use tokio::fs::metadata;

pub async fn exists(path: impl AsRef<Path>) -> bool {
    metadata(path).await.is_ok()
}

pub async fn is_dir(path: impl AsRef<Path>) -> bool {
    metadata(path).await.map(|m| m.is_dir()).unwrap_or(false)
}
