use chord_common::error::Error;
use chord_common::point::{async_trait, PointArg, PointRunner, PointRunnerFactory};
use chord_common::value::Map;

mod point;

pub struct PointRunnerFactoryDefault {
    point_config: Map,
}

impl PointRunnerFactoryDefault {
    pub async fn new(point_config: Map) -> Result<PointRunnerFactoryDefault, Error> {
        Ok(PointRunnerFactoryDefault { point_config })
    }
}

#[async_trait]
impl PointRunnerFactory for PointRunnerFactoryDefault {
    async fn create_runner(
        &self,
        kind: &str,
        arg: &dyn PointArg,
    ) -> Result<Box<dyn PointRunner>, Error> {
        point::create_kind_runner(kind, self.point_config.get(kind), arg).await
    }
}
