use chord_core::action::prelude::*;
use chord_core::action::CreateId;
use std::cell::RefCell;
use std::collections::HashMap;
use std::mem::replace;
use std::sync::{Arc, RwLock};

use crate::err;

pub struct ContextFactory {}

impl ContextFactory {
    pub async fn new(_: Option<Value>) -> Result<ContextFactory, Error> {
        Ok(ContextFactory {})
    }
}

struct CreateArgStruct<'o> {
    origin: &'o dyn CreateArg,
}

impl CreateArg for CreateArgStruct {
    fn id(&self) -> &dyn CreateId {
        todo!()
    }

    fn action(&self) -> &str {
        todo!()
    }

    fn args_raw(&self) -> &Value {
        todo!()
    }

    fn render_str(&self, text: &str) -> Result<Value, Error> {
        todo!()
    }

    fn is_static(&self, text: &str) -> bool {
        todo!()
    }

    fn factory(&self, action: &str) -> Option<&dyn Factory> {
        todo!()
    }
}

#[async_trait]
impl Factory for ContextFactory {
    async fn create(&self, arg: &dyn CreateArg) -> Result<Box<dyn Action>, Error> {
        let args_raw = arg.args_raw();
        let map = args_raw.as_object().unwrap();

        let mut action_vec = Vec::with_capacity(map.len());

        for (aid, fo) in map {
            let only = fo.as_object().unwrap().iter().last().unwrap();
            let func = only.0.as_str();

            let create_arg = CreateArgStruct { origin: arg };

            let action = arg
                .factory(func.into())
                .ok_or_else(|| err!("100", "unsupported action"))?
                .create(&create_arg)
                .await
                .map_err(|e| err!("100", "create error"))?;
            action_vec.push((aid.to_string(), action));
        }

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
