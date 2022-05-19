use crate::error::{Error, Result};
use std::fmt::Display;
use std::io::Write;
use terminal_menu::{button, menu, mut_menu, run};
use tokio::io::AsyncBufReadExt;
use tracing::warn;

pub struct InputReader;

impl InputReader {
    pub async fn request_user_input_string<M>(request_msg: &M) -> Result<String> where M: Display + ?Sized {
        InputReader::print_msg(request_msg)?;
        Self::wait_for_input().await
    }

    fn print_msg<M>(message: &M) -> Result<()> where M: Display + ?Sized {
        println!("  {}", message);
        print!("> ");
        std::io::stdout().flush().map_err(Into::into)
    }

    pub async fn ask_confirmation<M>(confirmation_msg: &M) -> bool
    where
        M: Display + ?Sized,
    {
        loop {
            let result = Self::request_user_input_string::<M>(confirmation_msg)
                .await
                .map(|answer| answer.to_lowercase());

            match result.as_ref().map(|v| v.as_ref()) {
                Ok("y" | "yes") => {
                    return true;
                }
                Ok("n" | "no") => {
                    return false;
                }
                _ => warn!("Invalid answer, try typing 'y' for yes or 'n' for no."),
            }
        }
    }

    pub async fn request_select_item_in_list<'a>(
        label: &str,
        list: impl Iterator<Item = &'a str>,
        custom_option: Option<&str>,
        // TODO: add separate func for go back
        can_go_back: bool,
    ) -> Option<usize> {
        let mut count: usize = 1;
        let mut menu_items = vec![terminal_menu::label(label)];
        for option in list {
            count += 1;
            menu_items.push(button(option));
        }
        if let Some(option) = custom_option {
            menu_items.push(button(option));
            count += 1;
        }
        if can_go_back {
            menu_items.push(button("Or Go back"));
        }

        let menu = menu(menu_items);
        run(&menu);
        let selected_index = mut_menu(&menu).selected_item_index();

        if can_go_back && selected_index == count {
            None
        } else {
            Some(selected_index - 1)
        }
    }

    async fn wait_for_input() -> Result<String> {
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
    }
}
