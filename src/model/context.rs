use std::borrow::Borrow;

use handlebars::Handlebars;

use crate::model::error::Error;
use crate::model::value::{Json};
use crate::model::helper::{NUM_HELPER, BOOL_HELPER};
use chrono::{Utc, DateTime};

pub type PointResult = std::result::Result<Json, Error>;

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
            end,
        }
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
    #[allow(dead_code)]
    pub fn result(&self) -> &Json {
        &self.result
    }
}

pub type PointResultInner = std::result::Result<PointResultStruct, Error>;

#[derive(Debug)]
pub enum CaseState {
    Ok,
    PointError(Error),
    PointFailure
}

impl CaseState {

    pub fn is_ok(&self) -> bool{
        self == Ok
    }
}

pub struct CaseResultStruct {
    result: Vec<(String,PointResultInner)>,
    id: usize,
    start: DateTime<Utc>,
    end: DateTime<Utc>,
    state: CaseState
}

impl CaseResultStruct {

    pub fn new(result:Vec<(String, PointResultInner)>,
               id: usize,
               start: DateTime<Utc>,
               end: DateTime<Utc>,
               state: CaseState
    ) -> CaseResultStruct {
        CaseResultStruct {
            result,id,start,end,state
        }
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
    pub fn state(&self) -> &CaseState {
        &self.state
    }
    pub fn result(&self) -> &Vec<(String, PointResultInner)> {
        &self.result
    }
}


pub type CaseResultInner = std::result::Result<CaseResultStruct, Error>;

pub enum TaskState {
    Ok,
    CaseError(Error),
    CaseFailure
}

pub struct TaskResultStruct {
    result: Vec<(usize, CaseResultInner)>,
    id: String,
    start: DateTime<Utc>,
    end: DateTime<Utc>,
    state: TaskState
}
impl TaskResultStruct{

    pub fn new(result:Vec<(usize, CaseResultInner)>,
               id: &str,
               start: DateTime<Utc>,
               end: DateTime<Utc>,
               state: TaskState
    ) -> TaskResultStruct {
        TaskResultStruct {
            result,
            id: String::from(id),
            start,
            end,
            state
        }
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

    #[allow(dead_code)]
    pub fn state(&self) -> &TaskState {
        &self.state
    }

    #[allow(dead_code)]
    pub fn result(&self) -> &Vec<(usize, CaseResultInner)> {
        &self.result
    }
}

pub type TaskResultInner = std::result::Result<TaskResultStruct, Error>;


pub trait PointContext{

    fn get_config_rendered(&self, path: Vec<&str>) -> Option<String>;

    fn get_config(&self) -> &Json;

    fn render(&self, text: &str) -> Result<String,Error>;
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
        let res = $crate::model::error::Error::new($code, $message);
        std::result::Result::Err(res)
    }}
}