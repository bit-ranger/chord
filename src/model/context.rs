use std::borrow::Borrow;

use handlebars::Handlebars;

use crate::model::error::Error;
use crate::model::value::{Json};
use crate::model::helper::{NUM_HELPER, BOOL_HELPER};

pub type BasicError = Error<()>;
pub type PointResult = std::result::Result<Json, BasicError>;
pub type CaseError = Error<Vec<(String, PointResult)>>;
pub type CaseResult = std::result::Result<Vec<(String, PointResult)>, CaseError>;
pub type TaskError = Error<Vec<CaseResult>>;
pub type TaskResult = std::result::Result<Vec<CaseResult>, TaskError>;

pub trait PointContext{

    fn get_config_rendered(&self, path: Vec<&str>) -> Option<String>;

    fn get_config(&self) -> &Json;

    fn render(&self, text: &str) -> Result<String,Error<()>>;
}

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


#[macro_export]
macro_rules! err {
    ($code:expr, $message:expr) => {{
        let res = $crate::model::context::BasicError::new($code, $message);
        std::result::Result::Err(res)
    }}
}