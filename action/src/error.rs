use std::fmt;
use std::fmt::{Debug, Display, Formatter};

use chord_core::value::json;

#[macro_export]
macro_rules! err {
    ($code:expr, $message:expr) => {{
        Box::new($crate::error::Error::new($code, $message))
    }};
}

#[derive(Clone)]
pub struct Error {
    code: String,
    message: String,
}

impl Error {
    pub fn new<C, M>(code: C, message: M) -> Error
    where
        C: Into<String>,
        M: Into<String>,
    {
        let c = code.into();
        let m = message.into();
        Error {
            code: c.clone(),
            message: m,
        }
    }

    #[allow(dead_code)]
    pub fn code(self: &Error) -> &str {
        return &self.code;
    }

    #[allow(dead_code)]
    pub fn message(self: &Error) -> &str {
        return &self.message;
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        Debug::fmt(&self, f)
    }
}

impl Debug for Error {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.write_str(
            json!({
                "code": self.code.clone(),
                "message": self.message.clone()
            })
            .to_string()
            .as_str(),
        )
    }
}

impl std::error::Error for Error {}
