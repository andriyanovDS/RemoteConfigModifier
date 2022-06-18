use crate::error::{Error, Result};
#[cfg(test)]
use mockall::automock;
use rustyline::error::ReadlineError;
use tracing::debug;

#[cfg_attr(test, automock)]
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
            #[cfg(unix)]
            ReadlineError::Errno(error) => Error::new(error.desc()),
            #[cfg(unix)]
            ReadlineError::Utf8Error => Error::new("Invalid characters in user input"),
            #[cfg(windows)]
            ReadlineError::WindowResize => Error::new("Unexpected error"),
            #[cfg(windows)]
            ReadlineError::Decode(_) => Error::new("Invalid characters in user input"),
            #[cfg(windows)]
            ReadlineError::SystemError(error) => {
                debug!("Process was interrupted.");
                Error::new("System error")
            }
            _ => Error::new("Unknown error happened while reading user input"),
        }
    }
}
