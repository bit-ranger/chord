use chord_common::point::{PointValue};
use chord_common::value::Json;

#[no_mangle]
pub fn create(args: Vec<&str>) -> PointValue {
    Ok(Json::String(format!("dynlib run: {:?}", args)))
}