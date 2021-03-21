use common::value::Json;
use std::num::ParseIntError;

pub struct PointError(common::error::Error);

impl PointError {

    pub fn new(code: &str, message: &str) -> PointError {
        PointError(common::error::Error::new(code, message))
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
        PointError::new("io", format!("{:?}", err.kind()).as_str())
    }
}

impl  From<common::value::JsonError> for PointError {
    fn from(err: common::value::JsonError) -> PointError {
        PointError::new("json", format!("{:?}", err).as_str())
    }
}


impl From<ParseIntError> for PointError {
    fn from(e: ParseIntError) -> PointError {
        PointError::new("ParseInt", e.to_string().as_str())
    }
}

pub fn to_common_value(point_value: PointValue) -> common::point::PointValue{
    return match point_value {
        Ok(pv) => Ok(pv),
        Err(e) => Err(e.common())
    }
}

pub type PointValue =  std::result::Result<Json, PointError>;




