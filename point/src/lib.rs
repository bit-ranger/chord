use chord_common::error::Error;
use chord_common::point::{async_trait, PointRunner, PointRunnerFactory, CreateArg};
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
    async fn create(
        &self,
        arg: &dyn CreateArg,
    ) -> Result<Box<dyn PointRunner>, Error> {
        let kind = arg.kind();
        point::create_kind_runner(kind, self.point_config.get(kind), arg).await
    }
}
