pub struct PortError(common::error::Error);

impl PortError {

    pub fn new(code: &str, message: &str) -> PortError {
        PortError(common::error::Error::new(code, message))
    }

    pub fn common(&self) -> common::error::Error{
        self.0.clone()
    }
}

impl From<common::error::Error> for PortError {
    fn from(err: common::error::Error) -> PortError {
        PortError(err)
    }
}

impl  From<std::io::Error> for PortError {
    fn from(err: std::io::Error) -> PortError {
        PortError::new("io", format!("{:?}", err.kind()).as_str())
    }
}

impl  From<common::value::JsonError> for PortError {
    fn from(err: common::value::JsonError) -> PortError {
        PortError::new("json", format!("{:?}", err).as_str())
    }
}

impl  From<csv::Error> for PortError {
    fn from(err: csv::Error) -> PortError {
        PortError::new("io", format!("{:?}", err.kind()).as_str())
    }
}




