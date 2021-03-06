use crate::model::error::ErrorStruct;

pub mod task;
pub mod case;
pub mod point;
pub mod app;
pub mod error;

pub type Json = serde_json::Value;
pub type Error = ErrorStruct;
