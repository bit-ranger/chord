use common::value::Json;
pub struct Error(common::error::Error);

impl Error {

    pub fn new(code: &str, message: &str) -> Error {
        Error(common::error::Error::new(code, message))
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


pub fn to_common_value(point_value: PointValue) -> common::point::PointValue{
    return match point_value {
        Ok(pv) => Ok(pv),
        Err(e) => Err(common::error::Error::new(e.0.get_code(), e.0.get_message()))
    }
}

pub type PointValue =  std::result::Result<Json, Error>;




