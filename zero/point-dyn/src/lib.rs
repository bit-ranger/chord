use chord_common::point::{PointValue};
use chord_common::value::Json;

#[no_mangle]
pub fn run(args: Vec<&str>) -> PointValue {
    Ok(Json::String(format!("dynlib run: {:?}", args)))
}