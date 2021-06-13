use std::borrow::Borrow;

use handlebars::Handlebars;

use chord::step::StepRunnerFactory;

use crate::model::helper::boolean::{ALL_HELPER, ANY_HELPER, BOOL_HELPER};
use crate::model::helper::number::NUM_HELPER;
use crate::model::helper::string::{contains, end_with, start_with, STR_HELPER, STR_SUB_HELPER};

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

        //handlebars-3.5.4/src/registry.rs:118
        handlebars.register_helper("all", Box::new(ALL_HELPER));
        handlebars.register_helper("any", Box::new(ANY_HELPER));
        handlebars.register_helper("num", Box::new(NUM_HELPER));
        handlebars.register_helper("bool", Box::new(BOOL_HELPER));
        handlebars.register_helper("str", Box::new(STR_HELPER));
        handlebars.register_helper("contains", Box::new(contains));
        handlebars.register_helper("str_start_with", Box::new(start_with));
        handlebars.register_helper("str_end_with", Box::new(end_with));
        handlebars.register_helper("str_sub", Box::new(STR_SUB_HELPER));

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

pub type RenderContext = handlebars::Context;
