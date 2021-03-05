use handlebars::{Handlebars};
use std::borrow::Borrow;

pub trait AppContext{


    fn get_handlebars<T>(&self) -> &Handlebars;
}

#[derive(Debug)]
pub struct AppContextStruct<'reg> {

    handlebars: Handlebars<'reg>
}

impl <'reg> AppContextStruct<'reg> {

    pub fn new() -> AppContextStruct<'reg>{
        AppContextStruct{
            handlebars: Handlebars::new()
        }
    }

}

impl <'reg> AppContext for AppContextStruct <'reg>{



    fn get_handlebars<T>(self: &AppContextStruct<'reg>) -> &Handlebars<'reg>
    {
        self.handlebars.borrow()
    }


}

