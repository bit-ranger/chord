use std::fmt::{Display, Formatter};
use std::fmt;
use std::sync::Arc;


#[macro_export]
macro_rules! err {
    ($code:expr, $message:expr) => {{
        let res = $crate::error::Error::new($code, $message);
        std::result::Result::Err(res)
    }}
}

#[macro_export]
macro_rules! perr {
    ($code:expr, $message:expr) => {{
        $crate::error::Error::new($code, $message)
    }}
}

#[macro_export]
macro_rules! cause {
    ($code:expr, $message:expr, $cause:expr) => {{
        let res = $crate::error::Error::cause($code, $message, $cause);
        std::result::Result::Err(res)
    }}
}

#[derive(Debug,Clone)]
pub struct Error

{
    code: String,
    message: String,
    cause: Option<Arc<dyn std::error::Error>>
}

impl  Error {

    pub fn new<C,M>(code: C, message: M) -> Error  where C: Into<String>, M: Into<String>{
        Error {
            code: code.into(),
            message: message.into(),
            cause: None
        }
    }

    pub fn cause<C,M,S>(code: C, message: M, cause: S) -> Error where C: Into<String>, M: Into<String>, S: std::error::Error+'static{
        Error {
            code: code.into(),
            message: message.into(),
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
                Some(c.as_ref())
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
       perr!("io", format!("{:?}", err))
    }
}

unsafe impl Send for Error
{
}

unsafe impl Sync for Error
{
}

