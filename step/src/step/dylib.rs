use chord_common::err;
use chord_common::error::Error;
use chord_common::step::{
    async_trait, CreateArg, RunArg, StepRunner, StepRunnerFactory, StepValue,
};
use chord_common::value::{Json, Map};
use dynamic_reload::{DynamicReload, Lib, PlatformName, Search, Symbol, UpdateState};
use std::cell::RefCell;
use std::ops::DerefMut;
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

        let step_runner_create: Symbol<fn(&dyn CreateArg) -> Result<(), Error>> =
            unsafe { lib.lib.get(b"create")? };

        step_runner_create(arg)?;

        Ok(Box::new(Runner { lib }))
    }
}

struct Runner {
    lib: Arc<Lib>,
}

unsafe impl Send for Runner {}

unsafe impl Sync for Runner {}

#[async_trait]
impl StepRunner for Runner {
    async fn run(&self, arg: &dyn RunArg) -> StepValue {
        let step_runner_run: Symbol<fn(&dyn RunArg) -> StepValue> =
            unsafe { self.lib.lib.get(b"run")? };
        step_runner_run(arg)
    }
}
