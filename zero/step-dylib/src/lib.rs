use chord_common::error::Error;
use chord_common::step::{CreateArg, RunArg, StepValue};
use chord_common::value::{Json, Map};
use lazy_static::lazy_static;
use std::ops::DerefMut;
use std::sync::Mutex;

lazy_static! {
    static ref CONTEXT: Mutex<Map> = Mutex::new(Map::new());
}

#[no_mangle]
pub fn create(arg: &dyn CreateArg) -> Result<(), Error> {
    let mut ctx = CONTEXT.lock().unwrap();
    let ctx = ctx.deref_mut();
    ctx.insert("create".into(), "1".into());
    println!("step_dylib create {}, {:?}", arg.id(), ctx);
    Ok(())
}

#[no_mangle]
pub fn run(arg: &dyn RunArg) -> StepValue {
    let mut ctx = CONTEXT.lock().unwrap();
    let ctx = ctx.deref_mut();
    ctx.insert("run".into(), "1".into());
    println!("step_dylib run {}, {:?}", arg.id(), ctx);

    Ok(Json::String(format!("step_dylib run {}", arg.id())))
}
