#[derive(Debug)]
pub struct Error {
    pub message: String,
}

pub type Result<T> = std::result::Result<T, Error>;

impl Error {
    pub fn new(message: &'static str) -> Self {
        Self { message: message.to_string() }
    }
}
