use std::sync::Arc;
use rlua;

#[derive(Debug)]
pub struct SimpleStringError {
  description: String
}

impl std::fmt::Display for SimpleStringError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.description)
    }
}

impl std::error::Error for SimpleStringError {
}

#[derive(Debug)]
pub enum Error {
  Lua {
    error: rlua::Error
  },
  IO {
    error: std::io::Error
  },
  SimpleString {
    error: SimpleStringError
  }
}

impl From<rlua::Error> for Error {
  fn from(error: rlua::Error) -> Self {
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
        description: description
      }
    }
  }
}

impl From<Error> for rlua::Error {
    fn from(error: Error) -> Self {
        match error {
            Error::Lua { error } => error,
            Error::IO { error } => rlua::Error::ExternalError(Arc::new(error)),
            Error::SimpleString { error } => rlua::Error::ExternalError(Arc::new(error))
        }
    }
}

