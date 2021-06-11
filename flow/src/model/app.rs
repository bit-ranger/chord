use std::borrow::Borrow;

use handlebars::Handlebars;

use chord::step::StepRunnerFactory;

use crate::model::helper::boolean::{ALL_HELPER, ANY_HELPER, BOOL_HELPER};
use crate::model::helper::number::NUM_HELPER;
use crate::model::helper::string::JOIN_HELPER;

pub trait Context: Sync + Send {
    fn get_handlebars(&self) -> &Handlebars;

    fn get_step_runner_factory(&self) -> &dyn StepRunnerFactory;
}

pub struct FlowContextStruct<'reg> {
    handlebars: Handlebars<'reg>,
    step_runner_factory: Box<dyn StepRunnerFactory>,
}

impl<'reg> FlowContextStruct<'reg> {
    pub fn new(step_runner_factory: Box<dyn StepRunnerFactory>) -> FlowContextStruct<'reg> {
        let mut handlebars = Handlebars::new();

        //C:/Users/bitranger/.cargo/registry/src/mirrors.ustc.edu.cn-61ef6e0cd06fb9b8/handlebars-3.5.4/src/registry.rs:118
        handlebars.register_helper("num", Box::new(NUM_HELPER));
        handlebars.register_helper("bool", Box::new(BOOL_HELPER));
        handlebars.register_helper("all", Box::new(ALL_HELPER));
        handlebars.register_helper("any", Box::new(ANY_HELPER));
        handlebars.register_helper("join", Box::new(JOIN_HELPER));

        FlowContextStruct {
            handlebars,
            step_runner_factory,
        }
    }
}

impl<'reg> Context for FlowContextStruct<'reg> {
    fn get_handlebars(self: &FlowContextStruct<'reg>) -> &Handlebars<'reg> {
        self.handlebars.borrow()
    }

    fn get_step_runner_factory(self: &FlowContextStruct<'reg>) -> &dyn StepRunnerFactory {
        self.step_runner_factory.as_ref()
    }
}

unsafe impl<'reg> Send for FlowContextStruct<'reg> {}

unsafe impl<'reg> Sync for FlowContextStruct<'reg> {}

pub type RenderContext = handlebars::Context;
