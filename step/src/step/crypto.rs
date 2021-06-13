use chord::step::{async_trait, CreateArg, RunArg, StepRunner, StepRunnerFactory, StepValue};
use chord::value::Value;
use chord::Error;
use chord::{err, rerr};

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
    let by = arg.config()["by"]
        .as_str()
        .ok_or(err!("010", "missing by"))?;

    let from = arg.config()["from"]
        .as_str()
        .map(|s| arg.render_str(s))
        .ok_or(err!("010", "missing from"))??;

    return match by {
        "md5" => {
            let digest = md5::compute(from);
            let digest = format!("{:x}", digest);
            return Ok(Value::String(digest));
        }
        _ => {
            rerr!("crypto", format!("unsupported {}", by))
        }
    };
}
