use common::value::Json;
use std::num::ParseIntError;

pub struct Error(common::error::Error);

impl Error {

    pub fn new(code: &str, message: &str) -> Error {
        Error(common::error::Error::new(code, message))
    }

    pub fn common(&self) -> common::error::Error{
        self.0.clone()
    }
}

impl From<common::error::Error> for Error {
    fn from(err: common::error::Error) -> Error {
        Error(err)
    }
}

impl  From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Error {
        Error::new("io", format!("{:?}", err.kind()).as_str())
    }
}

impl  From<common::value::JsonError> for Error{
    fn from(err: common::value::JsonError) -> Error {
        Error::new("json", format!("{:?}", err).as_str())
    }
}


impl From<ParseIntError> for Error{
    fn from(e: ParseIntError) -> Error {
        Error::new("ParseInt", e.to_string().as_str())
    }
}

pub fn to_common_value(point_value: PointValue) -> common::point::PointValue{
    return match point_value {
        Ok(pv) => Ok(pv),
        Err(e) => Err(e.common())
    }
}

pub type PointValue =  std::result::Result<Json, Error>;




