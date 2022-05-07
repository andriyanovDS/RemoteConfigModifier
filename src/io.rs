use crate::error::{Error, Result};
use std::fmt::Display;
use tokio::io::AsyncBufReadExt;
use tracing::info;

pub struct InputReader;

pub struct InputString(pub(crate) String);
struct InputUInt(usize);

impl InputReader {
    pub async fn request_user_input<R, M>(request_msg: &M) -> Result<R>
    where
        R: TryFrom<String, Error = Error>,
        M: Display + ?Sized,
    {
        info!("{}", request_msg);
        Self::wait_for_input().await
    }

    pub async fn ask_confirmation(confirmation_msg: &str) -> Result<bool> {
        let answer = Self::request_user_input::<InputString, str>(confirmation_msg).await?;
        match answer.0.to_lowercase().as_ref() {
            "y" | "yes" => Ok(true),
            "n" | "no" => Ok(false),
            _ => Err(Error::new("Unexpected answer")),
        }
    }

    pub async fn request_select_item_in_list<'a>(
        list: impl Iterator<Item = &'a str>,
    ) -> Result<Option<usize>> {
        let mut count: usize = 1;
        for (index, item) in list.enumerate() {
            count += 1;
            println!("{}) {}", index + 1, item);
        }
        println!("{}) Return back", count);
        println!();
        Self::wait_for_input::<InputUInt>()
            .await
            .and_then(|number| {
                let number = number.0;
                if number > count || count == 0 {
                    Err(Error::new("Unknown option"))
                } else if number == count {
                    Ok(None)
                } else {
                    Ok(Some(number - 1))
                }
            })
    }

    async fn wait_for_input<R>() -> Result<R>
    where
        R: TryFrom<String, Error = Error>,
    {
        let mut reader = tokio::io::BufReader::new(tokio::io::stdin());
        let mut buffer = String::new();
        reader
            .read_line(&mut buffer)
            .await
            .map_err(|_| Error::new("Failed to read input"))
            .map(move |_| {
                buffer.pop();
                buffer
            })
            .and_then(R::try_from)
    }
}

impl TryFrom<String> for InputString {
    type Error = Error;
    fn try_from(value: String) -> std::result::Result<Self, Self::Error> {
        Ok(Self(value))
    }
}

impl TryFrom<String> for InputUInt {
    type Error = Error;
    fn try_from(value: String) -> std::result::Result<Self, Self::Error> {
        value.parse::<usize>().map(InputUInt).map_err(|_| Error {
            message: format!("Failed to parse {} to the number", &value),
        })
    }
}
