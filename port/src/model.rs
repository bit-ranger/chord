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

impl  From<csv::Error> for Error {
    fn from(err: csv::Error) -> Error {
        Error::new("io", format!("{:?}", err.kind()).as_str())
    }
}




