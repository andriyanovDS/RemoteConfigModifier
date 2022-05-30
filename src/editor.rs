use crate::error::{Error, Result};
use rustyline::error::ReadlineError;
use tracing::debug;

pub trait Editor {
    fn read_line(&mut self) -> Result<String>;
}

impl Editor for rustyline::Editor<()> {
    fn read_line(&mut self) -> Result<String> {
        self.readline("> ").map_err(From::from)
    }
}

impl From<ReadlineError> for Error {
    fn from(error: ReadlineError) -> Self {
        match error {
            ReadlineError::Interrupted | ReadlineError::Eof => {
                debug!("Process was interrupted.");
                std::process::exit(0);
            }
            ReadlineError::Io(io_error) => Error::from(io_error),
            ReadlineError::Errno(error) => Error::new(error.desc()),
            ReadlineError::Utf8Error => Error::new("Invalid characters in user input"),
            _ => Error::new("Unknown error happened while reading user input"),
        }
    }
}
