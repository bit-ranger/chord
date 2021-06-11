use chord::err;
use chord::error::Error;
use chord::step::{async_trait, CreateArg, RunArg, StepRunner, StepRunnerFactory, StepValue};
use chord::value::json::{to_string, Json};
use dynamic_reload::{DynamicReload, Lib, PlatformName, Search, Symbol};
use std::sync::Arc;

pub struct Factory {}

impl Factory {
    pub async fn new(_: Option<Json>) -> Result<Factory, Error> {
        Ok(Factory {})
    }
}

#[async_trait]
impl StepRunnerFactory for Factory {
    async fn create(&self, arg: &dyn CreateArg) -> Result<Box<dyn StepRunner>, Error> {
        let dir = arg.config()["dir"]
            .as_str()
            .ok_or(err!("010", "missing dir"))?;

        let mut reload_handler = DynamicReload::new(Some(vec![dir]), Some(dir), Search::Default);

        let lib = reload_handler.add_library("step_dylib", PlatformName::Yes)?;

        let step_runner_create: Symbol<fn(&str, &str) -> Result<(), Error>> =
            unsafe { lib.lib.get(b"init")? };

        let config_str = to_string(arg.config())?;
        let config_str = arg.render(config_str.as_str())?;
        step_runner_create(arg.id().to_string().as_str(), config_str.as_str())?;

        Ok(Box::new(Runner { lib }))
    }
}

struct Runner {
    lib: Arc<Lib>,
}

#[async_trait]
impl StepRunner for Runner {
    async fn run(&self, arg: &dyn RunArg) -> StepValue {
        let step_runner_run: Symbol<fn(&str, &str) -> StepValue> =
            unsafe { self.lib.lib.get(b"run")? };

        let config_str = to_string(arg.config())?;
        let config_str = arg.render(config_str.as_str())?;
        step_runner_run(arg.id().to_string().as_str(), config_str.as_str())
    }
}
