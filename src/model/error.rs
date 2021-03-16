use std::fmt::{Display, Formatter, Debug};
use std::fmt;
use std::ops::Deref;

pub type Error<T> = ErrorStruct<T>;


#[derive(Debug, Clone)]
pub struct ErrorStruct<T>

{
    code: String,
    message: String,
    attach: Option<Box<T>>
}

impl<T> ErrorStruct<T>{

    pub fn new(code: &str, message: &str) -> ErrorStruct<T>{
        ErrorStruct{
            code: String::from(code),
            message: String::from(message),
            attach: None
        }
    }


    pub fn attach(code: &str, message: &str, attach: T) -> ErrorStruct<T>{
        ErrorStruct{
            code: String::from(code),
            message: String::from(message),
            attach: Some(Box::new(attach))
        }
    }

    pub fn get_attach(self: &ErrorStruct<T>) -> Option<&T>{
        match &self.attach {
            Some(x) => Some(Box::deref(x)),
            None => None,
        }
    }

    #[allow(dead_code)]
    pub fn get_code(self: &ErrorStruct<T>) -> &str{
        return &self.code
    }

    #[allow(dead_code)]
    pub fn get_message(self: &ErrorStruct<T>) -> &str{
        return &self.message
    }
}

impl <T> Display for ErrorStruct<T> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.write_fmt(format_args!("{} \"code\": \"{}\", \"message\": \"{}\" {}",
                                 "{", self.code, self.message, "}"))
    }
}


impl <T> From<std::io::Error> for Error<T> {
    fn from(err: std::io::Error) -> Error<T> {
        Error::new("io", format!("{:?}", err.kind()).as_str())
    }
}












