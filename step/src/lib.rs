use chord_common::error::Error;
use chord_common::step::{async_trait, StepRunner, StepRunnerFactory, CreateArg};
use chord_common::value::Map;

mod step;

pub struct StepRunnerFactoryDefault {
    step_config: Map,
}

impl StepRunnerFactoryDefault {
    pub async fn new(step_config: Map) -> Result<StepRunnerFactoryDefault, Error> {
        Ok(StepRunnerFactoryDefault { step_config })
    }
}

#[async_trait]
impl StepRunnerFactory for StepRunnerFactoryDefault {
    async fn create(
        &self,
        arg: &dyn CreateArg,
    ) -> Result<Box<dyn StepRunner>, Error> {
        let kind = arg.kind();
        step::create_kind_runner(kind, self.step_config.get(kind), arg).await
    }
}
