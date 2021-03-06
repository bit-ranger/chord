use std::fmt::{Display, Formatter};
use std::fmt;
pub type Error = ErrorStruct;


#[derive(Debug)]
pub struct ErrorStruct{
    code: String,
    message: String,
    desc: String
}

impl ErrorStruct{

    pub fn new(code: &str, message: &str) -> ErrorStruct{
        ErrorStruct{
            code: String::from(code),
            message: String::from(message),
            desc: format!("{} \"code\": \"{}\", \"message\": \"{}\" {}", "{", code, message, "}")
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
        f.write_str(&self.desc)
    }
}

impl std::error::Error for ErrorStruct {
    fn description(&self) -> &str {
        &self.desc
    }

    fn cause(self: &ErrorStruct) -> Option<&dyn std::error::Error> {
        None
    }
}

// impl From<ErrorStruct> for ErrorStruct {
//     fn from(err: ErrorStruct) -> ErrorStruct {
//         ErrorStruct::from_err("021", "invalid method", Box::new(err))
//     }
// }


// impl From<NoneError> for ErrorStruct {
//     fn from(err: NoneError) -> Self {
//         ErrorStruct::new("021", "invalid method")
//     }
// }




