use chord::value::{from_str, Map, Value};
use std::ffi::{CStr, CString};
use std::os::raw::c_char;

#[no_mangle]
pub fn run(req: *const c_char) -> *mut c_char {
    let req_cstr = unsafe { CStr::from_ptr(req) };

    let req: Value = from_str(req_cstr.to_str().unwrap()).unwrap();
    let id = req["id"].as_str().unwrap();
    let context = req["context"].as_object().unwrap();
    let args = req["args"].as_str().unwrap();
    let timeout = req["timeout"].as_u64().unwrap();
    println!(
        "cdylib_example run {}, {:?}, {}, {}",
        id, context, args, timeout
    );

    let mut obj = Map::new();
    obj.insert("run".into(), "1".into());
    let v = Value::Object(obj);
    let v = v.to_string();
    let v = CString::new(v).unwrap();
    v.into_raw()
}
