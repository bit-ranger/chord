use std::fmt::{Display, Formatter};
use std::fmt;

pub type Error = ErrorStruct;


#[derive(Debug, Clone)]
pub struct ErrorStruct

{
    code: String,
    message: String,
    cause: Option<Box<ErrorStruct>>
}

impl ErrorStruct{

    pub fn new(code: &str, message: &str) -> ErrorStruct{
        ErrorStruct{
            code: String::from(code),
            message: String::from(message),
            cause: None
        }
    }

    pub fn cause(code: &str, message: &str, cause: ErrorStruct) -> ErrorStruct{
        ErrorStruct{
            code: String::from(code),
            message: String::from(message),
            cause: Some(Box::new(cause))
        }
    }

    #[allow(dead_code)]
    pub fn get_code(self: &ErrorStruct) -> &str{
        return &self.code
    }

    #[allow(dead_code)]
    pub fn get_message(self: &ErrorStruct) -> &str{
        return &self.message
    }
}

impl  Display for ErrorStruct {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.write_fmt(format_args!("{} \"code\": \"{}\", \"message\": \"{}\" {}",
                                 "{", self.code, self.message, "}"))
    }
}


impl  From<std::io::Error> for ErrorStruct {
    fn from(err: std::io::Error) -> ErrorStruct {
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
