use chord::value::json;
use std::error::Error as StdError;
use std::fmt;
use std::fmt::{Debug, Display, Formatter};
use std::sync::Arc;

#[macro_export]
macro_rules! err {
    ($code:expr, $message:expr) => {{
        Box::new($crate::error::Error::new($code, $message))
    }};
}

#[macro_export]
macro_rules! cause {
    ($code:expr, $message:expr, $cause:expr) => {{
        Box::new($crate::error::Error::cause($code, $message, $cause))
    }};
}

#[derive(Clone)]
pub struct Error {
    code: String,
    message: String,
    cause: Arc<anyhow::Error>,
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
            cause: Arc::new(anyhow::Error::msg(c)),
        }
    }

    pub fn cause<C, M, E>(code: C, message: M, cause: E) -> Error
    where
        C: Into<String>,
        M: Into<String>,
        E: StdError + Send + Sync + 'static,
    {
        Error {
            code: code.into(),
            message: message.into(),
            cause: Arc::new(anyhow::Error::from(cause)),
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
        )?;
        f.write_str("\n")?;
        Debug::fmt(self.cause.as_ref(), f)
    }
}

impl std::error::Error for Error {}
