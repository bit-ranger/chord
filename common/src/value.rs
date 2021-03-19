pub type Json = serde_json::Value;
pub type Map = serde_json::Map<String, Json>;
pub type Number = serde_json::Number;
pub use serde_json::to_value as to_json;
use crate::error::Error;

impl  From<serde_json::error::Error> for Error{
    fn from(err: serde_json::error::Error) -> Error {
        Error::new("json", format!("{:?}", err).as_str())
    }
}