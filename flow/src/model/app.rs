use std::borrow::Borrow;

use handlebars::Handlebars;

use crate::model::helper::register;
use chord::action::Factory;

pub trait FlowApp: Sync + Send {
    fn get_handlebars(&self) -> &Handlebars;

    fn get_action_factory(&self) -> &dyn Factory;
}

pub struct FlowAppStruct<'reg> {
    handlebars: Handlebars<'reg>,
    action_factory: Box<dyn Factory>,
}

impl<'reg> FlowAppStruct<'reg> {
    pub fn new(action_factory: Box<dyn Factory>) -> FlowAppStruct<'reg> {
        let mut handlebars = Handlebars::new();
        handlebars.set_strict_mode(true);
        register(&mut handlebars);
        FlowAppStruct {
            handlebars,
            action_factory,
        }
    }
}

impl<'reg> FlowApp for FlowAppStruct<'reg> {
    fn get_handlebars(self: &FlowAppStruct<'reg>) -> &Handlebars<'reg> {
        self.handlebars.borrow()
    }

    fn get_action_factory(self: &FlowAppStruct<'reg>) -> &dyn Factory {
        self.action_factory.as_ref()
    }
}

pub type RenderContext = handlebars::Context;
