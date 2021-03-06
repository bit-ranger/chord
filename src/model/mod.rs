use crate::model::error::ErrorStruct;

pub mod app;
mod error;

pub type Json = serde_json::Value;
pub type Error = ErrorStruct;

pub type PointResult = std::result::Result<Json, Error>;
pub type CaseResult = std::result::Result<Vec<(String, PointResult)>, Error>;
pub type TaskResult = std::result::Result<Vec<CaseResult>, Error>;

pub trait PointContext{


    fn get_config_rendered(&self, path: Vec<&str>) -> Option<String>;

    fn get_config(&self) -> &Json;

    fn render(&self, text: &str) -> String;
}



