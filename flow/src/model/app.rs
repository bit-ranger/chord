use std::borrow::Borrow;

use handlebars::Handlebars;

use crate::model::helper::register;
use chord::action::Factory;
use chord::input::FlowParse;

pub trait FlowApp: Sync + Send {
    fn get_handlebars(&self) -> &Handlebars;

    fn get_action_factory(&self) -> &dyn Factory;

    fn get_flow_parse(&self) -> &dyn FlowParse;
}

pub struct FlowAppStruct<'reg> {
    handlebars: Handlebars<'reg>,
    action_factory: Box<dyn Factory>,
    flow_parse: Box<dyn FlowParse>,
}

impl<'reg> FlowAppStruct<'reg> {
    pub fn new(
        action_factory: Box<dyn Factory>,
        flow_parse: Box<dyn FlowParse>,
    ) -> FlowAppStruct<'reg> {
        let mut handlebars = Handlebars::new();
        handlebars.set_strict_mode(true);
        register(&mut handlebars);
        FlowAppStruct {
            handlebars,
            action_factory,
            flow_parse,
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

    fn get_flow_parse(&self) -> &dyn FlowParse {
        self.flow_parse.as_ref()
    }
}

pub type RenderContext = handlebars::Context;
