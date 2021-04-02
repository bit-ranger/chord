use common::value::Json;
use std::num::ParseIntError;
use backtrace::Backtrace;


#[macro_export]
macro_rules! err {
    ($code:expr, $message:expr) => {{
        let res = $crate::model::PointError::new($code, $message);
        std::result::Result::Err(res)
    }}
}

#[macro_export]
macro_rules! perr {
    ($code:expr, $message:expr) => {{
        $crate::model::PointError::new($code, $message)
    }}
}

pub struct PointError(common::error::Error);

impl PointError {

    pub fn new<C,M>(code: C, message: M) -> PointError where C: Into<String>, M: Into<String>{
        PointError(common::error::Error::new(code, message))
    }

    #[allow(dead_code)]
    pub fn trace<C,M>(code: C, message: M, bt: Backtrace) -> PointError where C: Into<String>, M: Into<String>{
        PointError(common::error::Error::new(code, format!("{} {:?}", message.into(), bt)))
    }

    pub fn common(&self) -> common::error::Error{
        self.0.clone()
    }
}

impl From<common::error::Error> for PointError {
    fn from(err: common::error::Error) -> PointError {
        PointError(err)
    }
}

impl  From<std::io::Error> for PointError {
    fn from(err: std::io::Error) -> PointError {
        PointError::new("io", format!("{:?}", err.kind()))
    }
}

impl  From<common::value::JsonError> for PointError {
    fn from(err: common::value::JsonError) -> PointError {
        PointError::new("json", format!("{:?}", err))
    }
}


impl From<ParseIntError> for PointError {
    fn from(e: ParseIntError) -> PointError {
        PointError::new("ParseInt", e.to_string())
    }
}

pub fn to_common_value(pt_value: PointValue) -> common::point::PointValue{
    return match pt_value {
        Ok(pv) => Ok(pv),
        Err(e) => Err(e.common())
    }
}

pub type PointValue =  std::result::Result<Json, PointError>;




