use std::fmt::{Display, Formatter};

pub mod container;
pub mod engine;
pub mod image;

pub enum Error {
    Host(String),
    Engine(String),
    Image(String),
    Container(String),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match &self {
            Error::Host(s) => f.write_str(s),
            Error::Engine(s) => f.write_str(s),
            Error::Image(s) => f.write_str(s),
            Error::Container(s) => f.write_str(s),
        }
    }
}
