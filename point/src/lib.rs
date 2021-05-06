use std::future::Future;
use std::pin::Pin;

use chord_common::point::{PointRunner, PointRunnerFactory};
use chord_common::error::Error;
use chord_common::value::Json;

mod point;

pub struct PointRunnerFactoryDefault;

impl PointRunnerFactoryDefault {
    pub async fn new() -> Result<PointRunnerFactoryDefault, Error>{
        Ok(PointRunnerFactoryDefault {})
    }
}

impl PointRunnerFactory for PointRunnerFactoryDefault {

    fn create_runner<'k>(&self, kind: &'k str, config: &'k Json) -> Pin<Box<dyn Future<Output=Result<Box<dyn PointRunner>, Error>> + Send + 'k>> {
        Box::pin(point::create_kind_runner(kind, config))
    }
}


unsafe impl Send for PointRunnerFactoryDefault
{
}

unsafe impl Sync for PointRunnerFactoryDefault
{
}





