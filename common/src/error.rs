use std::fmt::{Display, Formatter};
use std::fmt;
use std::sync::Arc;


#[derive(Debug,Clone)]
pub struct Error

{
    code: String,
    message: String,
    cause: Option<Arc<Box<dyn std::error::Error>>>
}

impl  Error {

    pub fn new<C,M>(code: C, message: M) -> Error  where C: Into<String>, M: Into<String>{
        Error {
            code: String::from(code.into()),
            message: String::from(message.into()),
            cause: None
        }
    }

    pub fn cause(code: &str, message: &str, cause: Box<dyn std::error::Error>) -> Error {
        Error {
            code: String::from(code),
            message: String::from(message),
            cause: Some(Arc::new(cause))
        }
    }

    #[allow(dead_code)]
    pub fn code(self: &Error) -> &str{
        return &self.code
    }

    #[allow(dead_code)]
    pub fn message(self: &Error) -> &str{
        return &self.message
    }

}

impl std::error::Error for Error {

    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self.cause{
            Some(c) =>{
                Some(c.as_ref().as_ref())
            },
            None => None
        }
    }
}

impl  Display for Error {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.write_str(format!("{} code: {}, message: {} {}",
                                 "{", self.code, self.message, "}").as_str())
    }
}


impl  From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Error {
        Error::new("io", format!("{:?}", err).as_str())
    }
}

impl  From<Box<dyn std::error::Error>> for Error {
    fn from(err: Box<dyn std::error::Error>) -> Error {
        Error::cause("std", err.to_string().as_str(), err)
    }
}

unsafe impl Send for Error
{
}

unsafe impl Sync for Error
{
}

#[macro_export]
macro_rules! err {
    ($code:expr, $message:expr) => {{
        let res = $crate::error::Error::new($code, $message);
        std::result::Result::Err(res)
    }}
}


#[macro_export]
macro_rules! cause {
    ($code:expr, $message:expr, $cause:expr) => {{
        let res = $crate::error::Error::cause($code, $message, std::boxed::Box::new($cause));
        std::result::Result::Err(res)
    }}
}