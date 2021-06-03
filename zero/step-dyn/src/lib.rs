use chord_common::step::{StepValue};
use chord_common::value::Json;

#[no_mangle]
pub fn run(args: Vec<&str>) -> StepValue {
    Ok(Json::String(format!("dynlib run: {:?}", args)))
}