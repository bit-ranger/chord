use chord::step::StepValue;
use chord::value::{from_str, Map, Value};
use chord::Error;
use lazy_static::lazy_static;
use std::ops::DerefMut;
use std::sync::Mutex;

lazy_static! {
    static ref CONTEXT: Mutex<Map> = Mutex::new(Map::new());
}

#[no_mangle]
pub fn init(id: &str, config: &str) -> Result<(), Error> {
    let config: Map = from_str(config)?;
    println!("fdylib create {}, {:?}", id, config);
    let mut ctx = CONTEXT.lock().unwrap();
    let ctx = ctx.deref_mut();
    ctx.insert("create".into(), "1".into());

    Ok(())
}

#[no_mangle]
pub fn run(id: &str, config: &str) -> StepValue {
    let config: Map = from_str(config)?;
    println!("fdylib run {}, {:?}", id, config);
    let mut ctx = CONTEXT.lock().unwrap();
    let ctx = ctx.deref_mut();
    ctx.insert("run".into(), "1".into());
    Ok(Value::String(format!("fdylib run {}", id)))
}
