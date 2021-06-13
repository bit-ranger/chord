use chord::err;
use chord::step::{async_trait, CreateArg, RunArg, StepRunner, StepRunnerFactory, StepValue};
use chord::value::Value;
use chord::Error;

pub struct Factory {}

impl Factory {
    pub async fn new(_: Option<Value>) -> Result<Factory, Error> {
        Ok(Factory {})
    }
}

#[async_trait]
impl StepRunnerFactory for Factory {
    async fn create(&self, _: &dyn CreateArg) -> Result<Box<dyn StepRunner>, Error> {
        Ok(Box::new(Runner {}))
    }
}

struct Runner {}

#[async_trait]
impl StepRunner for Runner {
    async fn run(&self, arg: &dyn RunArg) -> StepValue {
        run(arg).await
    }
}

async fn run(arg: &dyn RunArg) -> StepValue {
    let raw = arg.config()["raw"]
        .as_str()
        .map(|s| arg.render_str(s))
        .ok_or(err!("010", "missing raw"))??;
    let digest = md5::compute(raw);
    let digest = format!("{:x}", digest);
    return Ok(Value::String(digest));
}
