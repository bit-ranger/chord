use chord_common::err;
use chord_common::error::Error;
use chord_common::point::{async_trait, RunArg, PointRunner, PointValue, CreateArg};
use chord_common::value::Json;

struct UrlEncode {}

#[async_trait]
impl PointRunner for UrlEncode {
    async fn run(&self, arg: &dyn RunArg) -> PointValue {
        run(arg).await
    }
}

pub async fn create(_: Option<&Json>, _: &dyn CreateArg) -> Result<Box<dyn PointRunner>, Error> {
    Ok(Box::new(UrlEncode {}))
}

async fn run(arg: &dyn RunArg) -> PointValue {
    let raw = arg.config()["raw"]
        .as_str()
        .map(|s| arg.render(s))
        .ok_or(err!("010", "missing raw"))??;
    let digest = urlencoding::encode(raw.as_str());
    return Ok(Json::String(digest));
}
