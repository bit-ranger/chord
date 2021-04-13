use std::fmt::{Display, Formatter};
use std::fmt;
use std::sync::Arc;
use anyhow::anyhow;

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
    cause: Option<Arc<anyhow::Error>>
}

impl  Error {

    pub fn new<C,M>(code: C, message: M) -> Error  where C: Into<String>, M: Into<String>{
        let c = code.into();
        let m = message.into();
        Error {
            code: c,
            message: m,
            cause: None
        }
    }

    pub fn cause<C,M, E>(code: C, message: M, cause: E) -> Error where C: Into<String>, M: Into<String>, E: Into<anyhow::Error>{
        Error {
            code: code.into(),
            message: message.into(),
            cause: Some(Arc::new(cause.into()))
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

unsafe impl Send for Error {}

unsafe impl Sync for Error {}

// impl std::error::Error for Error {
//
//     fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
//         match &self.cause{
//             Some(c) =>{
//                 Some(c.root_cause())
//             },
//             None => None
//         }
//     }
// }

impl  Display for Error {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.write_str(format!("{} code: {}, message: {} {}",
                                 "{", self.code, self.message, "}").as_str())?;

        if let Some(cause) = &self.cause{
            f.write_str("\n")?;
            f.write_str(cause.to_string().as_str())?;
        }

        return Ok(());
    }
}


// impl  From<std::io::Error> for Error {
//     fn from(err: std::io::Error) -> Error {
//         Error::cause("io", err.to_string(), err)
//     }
// }

impl Into<anyhow::Error> for Error
{
    fn into(self) -> anyhow::Error {
        anyhow::Error::msg(format!("{}: {}", self.code, self.message))
    }
}

impl<E> From<E> for Error
    where
        E: std::error::Error + Send + Sync + 'static,
{
    fn from(error: E) -> Self {
        Error::cause("std", error.to_string(), error)
    }
}

