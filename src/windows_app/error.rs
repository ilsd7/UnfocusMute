use std::borrow::Cow;
use std::fmt;

pub type Result<T> = std::result::Result<T, AppError>;

#[derive(Debug)]
pub struct AppError {
    message: Cow<'static, str>,
}

impl AppError {
    fn new(message: impl Into<Cow<'static, str>>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl From<windows::core::Error> for AppError {
    fn from(error: windows::core::Error) -> Self {
        Self::new(error.to_string())
    }
}

pub fn message_error(message: impl Into<Cow<'static, str>>) -> AppError {
    AppError::new(message)
}

pub trait Context<T> {
    fn context(self, message: &'static str) -> Result<T>;
}

impl<T, E> Context<T> for std::result::Result<T, E>
where
    E: fmt::Display,
{
    fn context(self, message: &'static str) -> Result<T> {
        self.map_err(|error| message_error(format!("{message}: {error}")))
    }
}

impl<T> Context<T> for Option<T> {
    fn context(self, message: &'static str) -> Result<T> {
        self.ok_or_else(|| message_error(message))
    }
}
