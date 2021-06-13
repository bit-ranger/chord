use chord::step::{async_trait, CreateArg, RunArg, StepRunner, StepRunnerFactory, StepValue};
use chord::value::{from_str, to_string, Value};
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
        let config = arg.config().clone();
        let config_str = to_string(&config)?;
        let config_str = arg.render(config_str.as_str())?;
        let config: Value = from_str(config_str.as_str())?;
        return Ok(config);
    }
}
