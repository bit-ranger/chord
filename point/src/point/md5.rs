use chord_common::value::Json;
use chord_common::point::{PointArg, PointValue, PointRunner, Pin, Future};
use chord_common::error::Error;

struct Md5 {}

impl PointRunner for Md5 {

    fn run<'a>(&self, arg: &'a dyn PointArg) -> Pin<Box<dyn Future<Output=PointValue> + Send + 'a>> {
        Box::pin(run(arg))
    }
}

pub async fn create(_: &Json) -> Result<Box<dyn PointRunner>, Error>{
    Ok(Box::new(Md5 {}))
}


async fn run(pt_arg: &dyn PointArg) -> PointValue {
    let raw = pt_arg.config_rendered(vec!["raw"]).unwrap();
    let digest = md5::compute(raw);
    let digest = format!("{:x}", digest);
    return Ok(Json::String(digest));
}