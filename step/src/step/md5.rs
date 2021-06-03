use chord_common::err;
use chord_common::error::Error;
use chord_common::step::{async_trait, RunArg, StepRunner, StepValue, CreateArg};
use chord_common::value::Json;

struct Md5 {}

#[async_trait]
impl StepRunner for Md5 {
    async fn run(&self, arg: &dyn RunArg) -> StepValue {
        run(arg).await
    }
}

pub async fn create(_: Option<&Json>, _: &dyn CreateArg) -> Result<Box<dyn StepRunner>, Error> {
    Ok(Box::new(Md5 {}))
}

async fn run(arg: &dyn RunArg) -> StepValue {
    let raw = arg.config()["raw"]
        .as_str()
        .map(|s| arg.render(s))
        .ok_or(err!("010", "missing raw"))??;
    let digest = md5::compute(raw);
    let digest = format!("{:x}", digest);
    return Ok(Json::String(digest));
}
