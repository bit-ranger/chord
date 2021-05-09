use chord_common::error::Error;
use chord_common::point::{async_trait, PointArg, PointRunner, PointRunnerFactory};

mod point;

pub struct PointRunnerFactoryDefault;

impl PointRunnerFactoryDefault {
    pub async fn new() -> Result<PointRunnerFactoryDefault, Error> {
        Ok(PointRunnerFactoryDefault {})
    }
}

#[async_trait]
impl PointRunnerFactory for PointRunnerFactoryDefault {
    async fn create_runner(
        &self,
        kind: &str,
        arg: &dyn PointArg,
    ) -> Result<Box<dyn PointRunner>, Error> {
        point::create_kind_runner(kind, arg).await
    }
}
