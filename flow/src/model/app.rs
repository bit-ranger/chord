use std::borrow::Borrow;
use std::collections::HashMap;
use std::sync::Arc;

use handlebars::Handlebars;

use chord_core::action::Creator;

use crate::model::helper::register;

pub trait App: Sync + Send {
    fn get_handlebars(&self) -> &Handlebars;

    fn get_creator_map(&self) -> Arc<HashMap<String, Box<dyn Creator>>>;
}

pub struct AppStruct<'reg> {
    handlebars: Handlebars<'reg>,
    creator_map: Arc<HashMap<String, Box<dyn Creator>>>,
}

impl<'reg> AppStruct<'reg> {
    pub fn new(creator_map: HashMap<String, Box<dyn Creator>>) -> AppStruct<'reg> {
        let mut handlebars = Handlebars::new();
        handlebars.set_strict_mode(true);
        handlebars.register_escape_fn(handlebars::no_escape);
        register(&mut handlebars);
        AppStruct {
            handlebars,
            creator_map: Arc::new(creator_map),
        }
    }
}

impl<'reg> App for AppStruct<'reg> {
    fn get_handlebars(self: &AppStruct<'reg>) -> &Handlebars<'reg> {
        self.handlebars.borrow()
    }

    fn get_creator_map(self: &AppStruct<'reg>) -> Arc<HashMap<String, Box<dyn Creator>>> {
        self.creator_map.clone()
    }
}

pub type RenderContext = handlebars::Context;
