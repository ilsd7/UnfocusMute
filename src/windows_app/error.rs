use std::error::Error;
use std::fmt;

pub type Result<T> = std::result::Result<T, Box<dyn Error + Send + Sync + 'static>>;

#[derive(Debug)]
struct AppError(String);

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl Error for AppError {}

pub fn message_error(message: impl Into<String>) -> Box<dyn Error + Send + Sync + 'static> {
    Box::new(AppError(message.into()))
}

pub trait Context<T> {
    fn context(self, message: &'static str) -> Result<T>;
}

impl<T, E> Context<T> for std::result::Result<T, E>
where
    E: Error + Send + Sync + 'static,
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
