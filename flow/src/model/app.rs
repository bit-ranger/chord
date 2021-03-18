use std::borrow::Borrow;

use handlebars::Handlebars;

use common::point::PointRunner;

use crate::model::helper::{BOOL_HELPER, NUM_HELPER};

pub trait AppContext{

    fn get_handlebars(&self) -> &Handlebars;

    fn get_point_runner(&self) -> &dyn PointRunner;
}


pub struct AppContextStruct<'reg> {

    handlebars: Handlebars<'reg>,
    point_runner: Box<dyn PointRunner>
}

impl <'reg> AppContextStruct<'reg> {

    pub fn new(point_runner: Box<dyn PointRunner>) -> AppContextStruct<'reg>{
        let mut  handlebars = Handlebars::new();
        handlebars.register_helper("num", Box::new(NUM_HELPER));
        handlebars.register_helper("bool", Box::new(BOOL_HELPER));

        AppContextStruct{
            handlebars,
            point_runner
        }
    }

}

impl <'reg> AppContext for AppContextStruct <'reg>{

    fn get_handlebars(self: &AppContextStruct<'reg>) -> & Handlebars<'reg>
    {
        self.handlebars.borrow()
    }

    fn get_point_runner(self: &AppContextStruct<'reg>) -> & dyn PointRunner{
        self.point_runner.as_ref()
    }

}




