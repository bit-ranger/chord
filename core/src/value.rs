pub use serde::Deserialize;
pub use serde::Serialize;
pub use serde_json::error::Error;
pub use serde_json::from_reader;
pub use serde_json::from_slice;
pub use serde_json::from_str;
pub use serde_json::from_value;
pub use serde_json::json;
pub use serde_json::ser::to_string;
pub use serde_json::ser::to_string_pretty;
pub use serde_json::to_value;

pub type Value = serde_json::Value;
pub type Map = serde_json::Map<String, Value>;
pub type Number = serde_json::Number;

pub fn map_merge_deep(old: &Map, new: &Map) -> Map {
    let mut crt = old.clone();
    for (k, v) in new {
        let ov = crt.get(k);
        if let Some(ov) = ov {
            if let Value::Object(ov) = ov {
                if let Value::Object(nv) = v {
                    let mv = map_merge_deep(ov, nv);
                    crt.insert(k.clone(), Value::Object(mv));
                    continue;
                }
            }
        }
        crt.insert(k.clone(), v.clone());
    }
    crt
}
