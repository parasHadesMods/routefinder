use mlua;
use std::sync::Arc;

#[derive(Debug)]
pub struct SimpleStringError {
    description: String,
}

impl std::fmt::Display for SimpleStringError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.description)
    }
}

impl std::error::Error for SimpleStringError {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::Lua { error } => write!(f, "Lua error: {}", error),
            Error::IO { error } => write!(f, "IO error: {}", error),
            Error::SimpleString { error } => write!(f, "{}", error),
        }
    }
}

impl std::error::Error for Error {}

#[derive(Debug)]
pub enum Error {
    Lua { error: mlua::Error },
    IO { error: std::io::Error },
    SimpleString { error: SimpleStringError },
}

impl From<mlua::Error> for Error {
    fn from(error: mlua::Error) -> Self {
        Error::Lua { error: error }
    }
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Error::IO { error: error }
    }
}

impl From<String> for Error {
    fn from(description: String) -> Self {
        Error::SimpleString {
            error: SimpleStringError {
                description: description,
            },
        }
    }
}

impl From<Error> for mlua::Error {
    fn from(error: Error) -> Self {
        match error {
            Error::Lua { error } => error,
            Error::IO { error } => mlua::Error::ExternalError(Arc::new(error)),
            Error::SimpleString { error } => mlua::Error::ExternalError(Arc::new(error)),
        }
    }
}
