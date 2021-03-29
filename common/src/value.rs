pub type Json = serde_json::Value;
pub type Map = serde_json::Map<String, Json>;
pub type Number = serde_json::Number;
pub use serde_json::to_value as to_json;
pub use serde_json::from_str;
pub use serde_json::from_reader;
pub use serde_json::from_slice;
pub use serde_json::from_value;
pub use serde_json::error::Error as JsonError;
use crate::error::Error;
use crate::perr;

impl  From<serde_json::error::Error> for Error{
    fn from(err: serde_json::error::Error) -> Error {
       perr!("json", format!("{:?}", err))
    }
}