use async_std::future::Future;
use async_std::pin::Pin;
use chord_common::err;
use chord_common::error::Error;
use chord_common::point::{async_trait, PointArg, PointRunner, PointValue};

struct Dynlib {
    runner: Box<dyn PointRunner>,
}

#[async_trait]
impl PointRunner for Dynlib {
    async fn run(&self, arg: &dyn PointArg) -> PointValue {
        self.runner.run(arg).await
    }
}

pub async fn create(arg: &dyn PointArg) -> Result<Box<dyn PointRunner>, Error> {
    let path = arg.config()["path"]
        .as_str()
        .ok_or(err!("010", "missing path"))?;
    println!("loading dynlib {}", path);
    let lib = unsafe { libloading::Library::new(path)? };
    let point_runner_create: libloading::Symbol<
        fn(
            &dyn PointArg,
        ) -> Pin<Box<dyn Future<Output = Result<Box<dyn PointRunner>, Error>> + Send>>,
    > = unsafe { lib.get(b"create")? };
    let point_runner = point_runner_create(arg).await?;
    Ok(Box::new(Dynlib {
        runner: point_runner,
    }))
}
