use chord_common::value::Json;
use chord_common::point::{PointArg, PointValue, PointRunner, async_trait};
use chord_common::error::Error;
use chord_common::{err};

struct Md5 {}

#[async_trait]
impl PointRunner for Md5 {

    async fn run(&self, arg: &dyn PointArg) -> PointValue {
        run(arg).await
    }
}

pub async fn create(_: &dyn PointArg) -> Result<Box<dyn PointRunner>, Error>{
    Ok(Box::new(Md5 {}))
}


async fn run(arg: &dyn PointArg) -> PointValue {
    let raw = arg.config()["raw"].as_str()
        .map(|s|arg.render(s))
        .ok_or(err!("010", "missing raw"))??;
    let digest = md5::compute(raw);
    let digest = format!("{:x}", digest);
    return Ok(Json::String(digest));
}