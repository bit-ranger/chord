use std::borrow::Borrow;

use handlebars::Handlebars;

use crate::model::error::Error;
use crate::model::value::{Json};
use crate::model::helper::{NUM_HELPER, BOOL_HELPER};
use chrono::{Utc, DateTime};

pub type BasicError = Error<()>;
pub type PointResult = std::result::Result<Json, BasicError>;

pub struct PointResultStruct {
    result: Json,
    id: String,
    start: DateTime<Utc>,
    end: DateTime<Utc>,
}

impl PointResultStruct {

    pub fn new(result: Json,
           id: &str,
           start: DateTime<Utc>,
           end: DateTime<Utc>) -> PointResultStruct {
        PointResultStruct {
            result,
            id: String::from(id),
            start,
            end
        }
    }

    #[allow(dead_code)]
    pub fn result(&self) -> &Json {
        &self.result
    }
    #[allow(dead_code)]
    pub fn id(&self) -> &str {
        &self.id
    }
    #[allow(dead_code)]
    pub fn start(&self) -> DateTime<Utc> {
        self.start
    }
    #[allow(dead_code)]
    pub fn end(&self) -> DateTime<Utc> {
        self.end
    }
}

pub type PointErrorInner = Error<PointResultStruct>;
pub type PointResultInner = std::result::Result<PointResultStruct, PointErrorInner>;


pub struct CaseResultStruct {
    result: Vec<PointResultInner>,
    id: usize,
    start: DateTime<Utc>,
    end: DateTime<Utc>,
}

impl CaseResultStruct {

    pub fn new(result:Vec<PointResultInner>,
           id: usize,
           start: DateTime<Utc>,
           end: DateTime<Utc>) -> CaseResultStruct {
        CaseResultStruct {
            result,id,start,end
        }
    }

    #[allow(dead_code)]
    pub fn result(&self) -> &Vec<PointResultInner> {
        &self.result
    }
    #[allow(dead_code)]
    pub fn id(&self) -> usize {
        self.id
    }
    #[allow(dead_code)]
    pub fn start(&self) -> DateTime<Utc> {
        self.start
    }
    #[allow(dead_code)]
    pub fn end(&self) -> DateTime<Utc> {
        self.end
    }
}


pub type CaseError = Error<CaseResultStruct>;
pub type CaseResult = std::result::Result<CaseResultStruct, CaseError>;


pub struct TaskResultStruct {
    result: Vec<CaseResult>,
    id: String,
    start: DateTime<Utc>,
    end: DateTime<Utc>,
}
impl TaskResultStruct{

    pub fn new(result:Vec<CaseResult>,
           id: &str,
           start: DateTime<Utc>,
           end: DateTime<Utc>) -> TaskResultStruct {
        TaskResultStruct {
            result,
            id: String::from(id),
            start,
            end
        }
    }

    #[allow(dead_code)]
    pub fn result(&self) -> &Vec<CaseResult> {
        &self.result
    }

    #[allow(dead_code)]
    pub fn id(&self) -> &str {
        &self.id
    }

    #[allow(dead_code)]
    pub fn start(&self) -> DateTime<Utc> {
        self.start
    }

    #[allow(dead_code)]
    pub fn end(&self) -> DateTime<Utc> {
        self.end
    }
}

pub type TaskError = Error<TaskResultStruct>;
pub type TaskResult = std::result::Result<TaskResultStruct, TaskError>;


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