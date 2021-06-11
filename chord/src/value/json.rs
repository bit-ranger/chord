pub type Json = serde_json::Value;
pub type Map = serde_json::Map<String, Json>;
pub type Number = serde_json::Number;

pub use serde_json::error::Error;
pub use serde_json::from_reader;
pub use serde_json::from_slice;
pub use serde_json::from_str;
pub use serde_json::from_value;
pub use serde_json::json;
pub use serde_json::ser::to_string;
pub use serde_json::ser::to_string_pretty;
pub use serde_json::to_value as to_json;
