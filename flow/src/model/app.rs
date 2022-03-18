use std::borrow::Borrow;
use std::collections::HashMap;

use handlebars::Handlebars;

use chord_core::action::Factory;

use crate::model::helper::register;

pub trait App: Sync + Send {
    fn get_handlebars(&self) -> &Handlebars;

    fn get_action_factory(&self, name: &str) -> Option<&dyn Factory>;
}

pub struct AppStruct<'reg> {
    handlebars: Handlebars<'reg>,
    factory_map: HashMap<String, Box<dyn Factory>>,
}

impl<'reg> AppStruct<'reg> {
    pub fn new(action_factory: HashMap<String, Box<dyn Factory>>) -> AppStruct<'reg> {
        let mut handlebars = Handlebars::new();
        handlebars.set_strict_mode(true);
        handlebars.register_escape_fn(handlebars::no_escape);
        register(&mut handlebars);
        AppStruct {
            handlebars,
            factory_map: action_factory,
        }
    }
}

impl<'reg> App for AppStruct<'reg> {
    fn get_handlebars(self: &AppStruct<'reg>) -> &Handlebars<'reg> {
        self.handlebars.borrow()
    }

    fn get_action_factory(self: &AppStruct<'reg>, name: &str) -> Option<&dyn Factory> {
        self.factory_map.get(name).map(|f| f.as_ref())
    }
}

pub type RenderContext = handlebars::Context;
