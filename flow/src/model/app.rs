use std::borrow::Borrow;

use handlebars::Handlebars;

use crate::model::helper::register;
use chord::action::Factory;
use chord::input::FlowParse;

pub trait Context: Sync + Send {
    fn get_handlebars(&self) -> &Handlebars;

    fn get_action_factory(&self) -> &dyn Factory;

    fn get_flow_parse(&self) -> &dyn FlowParse;
}

pub struct FlowContextStruct<'reg> {
    handlebars: Handlebars<'reg>,
    action_factory: Box<dyn Factory>,
    flow_parse: Box<dyn FlowParse>,
}

impl<'reg> FlowContextStruct<'reg> {
    pub fn new(
        action_factory: Box<dyn Factory>,
        flow_parse: Box<dyn FlowParse>,
    ) -> FlowContextStruct<'reg> {
        let mut handlebars = Handlebars::new();
        register(&mut handlebars);
        FlowContextStruct {
            handlebars,
            action_factory,
            flow_parse,
        }
    }
}

impl<'reg> Context for FlowContextStruct<'reg> {
    fn get_handlebars(self: &FlowContextStruct<'reg>) -> &Handlebars<'reg> {
        self.handlebars.borrow()
    }

    fn get_action_factory(self: &FlowContextStruct<'reg>) -> &dyn Factory {
        self.action_factory.as_ref()
    }

    fn get_flow_parse(&self) -> &dyn FlowParse {
        self.flow_parse.as_ref()
    }
}

pub type RenderContext = handlebars::Context;
