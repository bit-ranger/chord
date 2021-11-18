pub mod container;
pub mod engine;
pub mod image;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("invalid url {0}")]
    Url(String),
    #[error("error with status {0}")]
    Status(u16),
    #[error("{0}")]
    Io(String),
    #[error("container error: {0}")]
    Container(String),
}
