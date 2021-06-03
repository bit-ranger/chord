use std::borrow::Borrow;

use handlebars::Handlebars;

use chord_common::step::StepRunnerFactory;

use crate::model::helper::{ALL_HELPER, ANY_HELPER, BOOL_HELPER, NUM_HELPER};

pub trait FlowContext: Sync + Send {
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
        handlebars.register_helper("num", Box::new(NUM_HELPER));
        handlebars.register_helper("bool", Box::new(BOOL_HELPER));
        handlebars.register_helper("all", Box::new(ALL_HELPER));
        handlebars.register_helper("any", Box::new(ANY_HELPER));

        FlowContextStruct {
            handlebars,
            step_runner_factory,
        }
    }
}

impl<'reg> FlowContext for FlowContextStruct<'reg> {
    fn get_handlebars(self: &FlowContextStruct<'reg>) -> &Handlebars<'reg> {
        self.handlebars.borrow()
    }

    fn get_step_runner_factory(self: &FlowContextStruct<'reg>) -> &dyn StepRunnerFactory {
        self.step_runner_factory.as_ref()
    }
}

unsafe impl<'reg> Send for FlowContextStruct<'reg> {}

unsafe impl<'reg> Sync for FlowContextStruct<'reg> {}
