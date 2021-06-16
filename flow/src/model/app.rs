use std::borrow::Borrow;

use handlebars::Handlebars;

use crate::model::helper::register;
use chord::step::StepRunnerFactory;

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
        register(&mut handlebars);
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
