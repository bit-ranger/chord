use chord_common::error::Error;
use chord_common::point::{async_trait, PointArg, PointRunner, PointValue};
use chord_common::value::Json;

#[no_mangle]
pub async fn create(arg: &dyn PointArg) -> Result<Box<dyn PointRunner>, Error> {
    Ok(Box::new(PointDyn {}))
}

struct PointDyn {}

#[async_trait]
impl PointRunner for PointDyn {
    async fn run(&self, arg: &dyn PointArg) -> PointValue {
        Ok(Json::String("hello point dyn".to_owned()))
    }
}
