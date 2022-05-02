use std::fmt::Display;
use tokio::io::AsyncBufReadExt;
use crate::error::{Error, Result};

pub struct InputReader;

pub struct InputString(pub(crate) String);

impl InputReader {
    pub async fn request_user_input<R, M>(
        request_msg: &M
    ) -> Result<R>
        where
            R: TryFrom<String, Error=Error>,
            M: Display + ?Sized
    {
        println!("{}", request_msg);
        let mut reader = tokio::io::BufReader::new(tokio::io::stdin());
        let mut buffer = String::new();
        reader.read_line(&mut buffer).await
            .map_err(|_| Error::new("Failed to read input"))
            .map(move |_| {
                buffer.pop();
                buffer
            })
            .and_then(R::try_from)
    }

    pub async fn ask_confirmation(confirmation_msg: &str) -> Result<bool> {
        let answer = Self::request_user_input::<InputString, str>(confirmation_msg).await?;
        match answer.0.as_ref() {
            "y" | "yes" => Ok(true),
            "n" | "no" => Ok(false),
            _ => Err(Error::new("Unexpected answer"))
        }
    }
}

impl TryFrom<String> for InputString {
    type Error = Error;

    fn try_from(value: String) -> std::result::Result<Self, Self::Error> {
        Ok(Self(value))
    }
}
