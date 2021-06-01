use chord_common::err;
use chord_common::error::Error;
use chord_common::point::{async_trait, PointArg, PointRunner, PointValue};
use chord_common::value::Json;

struct UrlDecode {}

#[async_trait]
impl PointRunner for UrlDecode {
    async fn run(&self, arg: &dyn PointArg) -> PointValue {
        run(arg).await
    }
}

pub async fn create(_: Option<&Json>, _: &dyn PointArg) -> Result<Box<dyn PointRunner>, Error> {
    Ok(Box::new(UrlDecode {}))
}

async fn run(arg: &dyn PointArg) -> PointValue {
    let raw = arg.config()["raw"]
        .as_str()
        .map(|s| arg.render(s))
        .ok_or(err!("010", "missing raw"))??;
    let digest = urlencoding::decode(raw.as_str())?;
    return Ok(Json::String(digest));
}
