use crate::error::{Error, Result};
use std::fmt::Display;
use std::io::Write;
use terminal_menu::{button, menu, mut_menu, run};
use tokio::io::AsyncBufReadExt;

pub struct InputReader;

pub struct InputString(pub(crate) String);
struct InputUInt(usize);

impl InputReader {
    pub async fn request_user_input<R, M>(request_msg: &M) -> Result<R>
    where
        R: TryFrom<String, Error = Error>,
        M: Display + ?Sized,
    {
        println!("{}", request_msg);
        print!("> ");
        std::io::stdout().flush()?;
        Self::wait_for_input().await
    }

    pub async fn ask_confirmation<M>(confirmation_msg: &M) -> Result<bool>
    where
        M: Display + ?Sized,
    {
        let answer = Self::request_user_input::<InputString, M>(confirmation_msg).await?;
        match answer.0.to_lowercase().as_ref() {
            "y" | "yes" => Ok(true),
            "n" | "no" => Ok(false),
            _ => Err(Error::new("Unexpected answer")),
        }
    }

    pub async fn request_select_item_in_list<'a>(
        list: impl Iterator<Item = &'a str>,
        custom_option: Option<&str>,
    ) -> Option<usize> {
        let mut count: usize = 1;
        let mut buttons = Vec::new();
        for option in list {
            count += 1;
            buttons.push(button(option));
        }
        if let Some(option) = custom_option {
            buttons.push(button(option));
            count += 1;
        }
        buttons.push(button("Or Go back"));

        let menu = menu(buttons);
        run(&menu);
        let selected_index = mut_menu(&menu).selected_item_index();

        if selected_index == count - 1 {
            None
        } else {
            Some(selected_index)
        }
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
