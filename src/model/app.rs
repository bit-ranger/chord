use std::borrow::Borrow;

use handlebars::Handlebars;

use crate::model::helper::{BOOL_HELPER, NUM_HELPER};

pub trait AppContext{


    fn get_handlebars(&self) -> &Handlebars;
}

#[derive(Debug)]
pub struct AppContextStruct<'reg> {

    handlebars: Handlebars<'reg>
}

impl <'reg> AppContextStruct<'reg> {

    pub fn new() -> AppContextStruct<'reg>{
        let mut  handlebars = Handlebars::new();
        handlebars.register_helper("num", Box::new(NUM_HELPER));
        handlebars.register_helper("bool", Box::new(BOOL_HELPER));

        AppContextStruct{
            handlebars
        }
    }

}

impl <'reg> AppContext for AppContextStruct <'reg>{

    fn get_handlebars(self: &AppContextStruct<'reg>) -> &Handlebars<'reg>
    {
        self.handlebars.borrow()
    }


}




