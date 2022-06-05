use std::borrow::Borrow;
use std::collections::HashMap;
use std::sync::Arc;

use handlebars::Handlebars;

use chord_core::action::Player;

use crate::model::helper::register;

pub trait App: Sync + Send {
    fn get_handlebars(&self) -> &Handlebars;

    fn get_action_map(&self) -> Arc<HashMap<String, Box<dyn Player>>>;
}

pub struct AppStruct<'reg> {
    handlebars: Handlebars<'reg>,
    action_map: Arc<HashMap<String, Box<dyn Player>>>,
}

impl<'reg> AppStruct<'reg> {
    pub fn new(action_map: HashMap<String, Box<dyn Player>>) -> AppStruct<'reg> {
        let mut handlebars = Handlebars::new();
        handlebars.set_strict_mode(true);
        handlebars.register_escape_fn(handlebars::no_escape);
        register(&mut handlebars);
        AppStruct {
            handlebars,
            action_map: Arc::new(action_map),
        }
    }
}

impl<'reg> App for AppStruct<'reg> {
    fn get_handlebars(self: &AppStruct<'reg>) -> &Handlebars<'reg> {
        self.handlebars.borrow()
    }

    fn get_action_map(self: &AppStruct<'reg>) -> Arc<HashMap<String, Box<dyn Player>>> {
        self.action_map.clone()
    }
}

pub type RenderContext = handlebars::Context;
