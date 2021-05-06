use std::borrow::Borrow;

use handlebars::Handlebars;

use chord_common::point::{PointRunnerFactory};

use crate::model::helper::{BOOL_HELPER, NUM_HELPER, ALL_HELPER, ANY_HELPER};

pub trait FlowContext: Sync+Send{

    fn get_handlebars(&self) -> &Handlebars;

    fn get_point_runner_factory(&self) -> &dyn PointRunnerFactory;
}


pub struct FlowContextStruct<'reg> {

    handlebars: Handlebars<'reg>,
    point_runner_factory: Box<dyn PointRunnerFactory>
}

impl <'reg> FlowContextStruct<'reg> {

    pub fn new(point_runner_factory: Box<dyn PointRunnerFactory>) -> FlowContextStruct<'reg>{
        let mut  handlebars = Handlebars::new();
        handlebars.register_helper("num", Box::new(NUM_HELPER));
        handlebars.register_helper("bool", Box::new(BOOL_HELPER));
        handlebars.register_helper("all", Box::new(ALL_HELPER));
        handlebars.register_helper("any", Box::new(ANY_HELPER));

        FlowContextStruct {
            handlebars,
            point_runner_factory
        }
    }

}

impl <'reg> FlowContext for FlowContextStruct<'reg>{

    fn get_handlebars(self: &FlowContextStruct<'reg>) -> & Handlebars<'reg>
    {
        self.handlebars.borrow()
    }

    fn get_point_runner_factory(self: &FlowContextStruct<'reg>) -> &dyn PointRunnerFactory{
        self.point_runner_factory.as_ref()
    }

}

unsafe impl<'reg> Send for FlowContextStruct<'reg>
{
}

unsafe impl<'reg> Sync for FlowContextStruct<'reg>
{
}



