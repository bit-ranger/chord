use chord_common::point::{PointRunner, PointRunnerFactory, async_trait, PointArg};
use chord_common::error::Error;

mod point;

pub struct PointRunnerFactoryDefault;

impl PointRunnerFactoryDefault {
    pub async fn new() -> Result<PointRunnerFactoryDefault, Error>{
        Ok(PointRunnerFactoryDefault {})
    }
}

#[async_trait]
impl PointRunnerFactory for PointRunnerFactoryDefault {

    async fn create_runner(&self, kind: &str, arg: &dyn PointArg) ->  Result<Box<dyn PointRunner>, Error>{
        point::create_kind_runner(kind, arg).await
    }
}




