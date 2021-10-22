use chord::value::{from_str, Map, Value};
use chord::Error;

#[no_mangle]
pub fn run(req: &str) -> Result<String, Error> {
    let req: Value = from_str(req).unwrap();
    let id = req["id"].as_str().unwrap();
    let context = req["context"].as_object().unwrap();
    let args = req["args"].as_object().unwrap();
    let timeout = req["timeout"].as_u64().unwrap();
    println!(
        "dylib_example run {}, {:?}, {:?}, {}",
        id, context, args, timeout
    );
    let mut obj = Map::new();
    obj.insert("run".into(), "1".into());
    let v = Value::Object(obj);
    let v = v.to_string();
    Ok(v)
}
