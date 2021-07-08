use std::error::Error as StdError;
use std::fmt;
use std::fmt::{Debug, Display, Formatter};
use std::sync::Arc;

use crate::value::json;

#[macro_export]
macro_rules! err {
    ($code:expr, $message:expr) => {{
        $crate::Error::new($code, $message)
    }};
}

#[macro_export]
macro_rules! cause {
    ($code:expr, $message:expr, $cause:expr) => {{
        $crate::Error::cause($code, $message, $cause)
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

impl Debug for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
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

impl<E> From<E> for Error
where
    E: StdError + Send + Sync + 'static,
{
    fn from(error: E) -> Self {
        Error::cause("std", error.to_string(), error)
    }
}

impl AsRef<dyn StdError + Send + Sync> for Error {
    fn as_ref(&self) -> &(dyn StdError + Send + Sync + 'static) {
        self.cause.as_ref().as_ref()
    }
}

impl AsRef<dyn StdError> for Error {
    fn as_ref(&self) -> &(dyn StdError + 'static) {
        self.cause.as_ref().as_ref()
    }
}
