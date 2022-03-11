use std::cell::RefCell;
use std::collections::HashMap;
use std::mem::replace;
use std::sync::{Arc, RwLock};

use chord_core::action::prelude::*;

pub struct ContextFactory {}

impl ContextFactory {
    pub async fn new(_: Option<Value>) -> Result<ContextFactory, Error> {
        Ok(ContextFactory {})
    }
}

#[async_trait]
impl Factory for ContextFactory {
    async fn create(&self, _: &dyn CreateArg) -> Result<Box<dyn Action>, Error> {
        Ok(Box::new(Context {}))
    }
}

struct Context {}

#[async_trait]
impl Action for Context {
    async fn run(&self, arg: &mut dyn RunArg) -> Result<Box<dyn Scope>, Error> {
        Ok(Box::new(arg.args()?))
    }
}
