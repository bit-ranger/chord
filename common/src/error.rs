use std::fmt::{Display, Formatter};
use std::fmt;



#[derive(Debug, Clone)]
pub struct Error

{
    code: String,
    message: String,
    cause: Option<Box<Error>>
}

impl Error {

    pub fn new(code: &str, message: &str) -> Error {
        Error {
            code: String::from(code),
            message: String::from(message),
            cause: None
        }
    }

    pub fn cause(code: &str, message: &str, cause: Error) -> Error {
        Error {
            code: String::from(code),
            message: String::from(message),
            cause: Some(Box::new(cause))
        }
    }

    #[allow(dead_code)]
    pub fn get_code(self: &Error) -> &str{
        return &self.code
    }

    #[allow(dead_code)]
    pub fn get_message(self: &Error) -> &str{
        return &self.message
    }
}

impl  Display for Error {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.write_fmt(format_args!("{} \"code\": \"{}\", \"message\": \"{}\" {}",
                                 "{", self.code, self.message, "}"))
    }
}


impl  From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Error {
        Error::new("io", format!("{:?}", err.kind()).as_str())
    }
}

#[macro_export]
macro_rules! err {
    ($code:expr, $message:expr) => {{
        let res = $crate::error::Error::new($code, $message);
        std::result::Result::Err(res)
    }}
}
