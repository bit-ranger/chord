use chord_common::point::{PointRunner, PointRunnerFactory, async_trait};
use chord_common::error::Error;
use chord_common::value::Json;

mod point;

pub struct PointRunnerFactoryDefault;

impl PointRunnerFactoryDefault {
    pub async fn new() -> Result<PointRunnerFactoryDefault, Error>{
        Ok(PointRunnerFactoryDefault {})
    }
}

#[async_trait]
impl PointRunnerFactory for PointRunnerFactoryDefault {

    async fn create_runner(&self, kind: &str, config: &Json) ->  Result<Box<dyn PointRunner>, Error>{
        point::create_kind_runner(kind, config).await
    }
}




