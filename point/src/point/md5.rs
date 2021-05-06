use chord_common::value::Json;
use chord_common::point::{PointArg, PointValue, PointRunner, async_trait};
use chord_common::error::Error;

struct Md5 {}

#[async_trait]
impl PointRunner for Md5 {

    async fn run(&self, arg: &dyn PointArg) -> PointValue {
        run(arg).await
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