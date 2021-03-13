use std::fmt::{Display, Formatter};
use std::fmt;
pub type Error = ErrorStruct;


#[derive(Debug, Clone)]
pub struct ErrorStruct{
    code: String,
    message: String
}

impl ErrorStruct{

    pub fn new(code: &str, message: &str) -> ErrorStruct{
        ErrorStruct{
            code: String::from(code),
            message: String::from(message)
        }
    }

    // pub fn from_err(code: &str, message: &str, err: Box<dyn std::error::Error>) -> ErrorStruct{
    //     ErrorStruct{
    //         code: String::from(code),
    //         message: String::from(message),
    //         cause: Some(err)
    //     }
    // }

}

impl Display for ErrorStruct {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.write_fmt(format_args!("{} \"code\": \"{}\", \"message\": \"{}\" {}",
                                 "{", self.code, self.message, "}"))
    }
}

impl std::error::Error for ErrorStruct {
    fn description(&self) -> &str {
        &self.code
    }

    fn cause(self: &ErrorStruct) -> Option<&dyn std::error::Error> {
        None
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Error {
        Error::new("io", format!("{:?}", err.kind()).as_str())
    }
}


// impl From<NoneError> for ErrorStruct {
//     fn from(err: NoneError) -> Self {
//         ErrorStruct::new("021", "invalid method")
//     }
// }




