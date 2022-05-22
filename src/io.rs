use crate::error::{Error, Result};
use color_eyre::owo_colors::OwoColorize;
use std::fmt::Display;
use std::io::Write;
use terminal_menu::{button, menu, mut_menu, run};
use tokio::io::AsyncBufReadExt;
use tracing::warn;

pub struct InputReader;

impl InputReader {
    pub async fn request_user_input_string<M>(request_msg: &M) -> Result<String>
    where
        M: Display + ?Sized,
    {
        InputReader::print_msg(request_msg)?;
        Self::wait_for_input().await
    }

    fn print_msg<M>(message: &M) -> Result<()>
    where
        M: Display + ?Sized,
    {
        println!("  {}", message);
        print!("> ");
        std::io::stdout().flush().map_err(Into::into)
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
    ) -> Option<usize> {
        let label = label.green().to_string();
        let mut menu_items: Vec<_> = std::iter::once(terminal_menu::label(""))
            .chain(std::iter::once(terminal_menu::label(&label)))
            .chain(list.map(button))
            .collect();

        if let Some(option) = custom_option {
            menu_items.push(button(option));
        }
        menu_items.push(button("Go back"));

        let items_len = menu_items.len();
        let menu = menu(menu_items);
        run(&menu);
        let menu = mut_menu(&menu);
        let selected_index = menu.selected_item_index();

        if selected_index == items_len - 1 {
            None
        } else {
            let selected_item_name = menu.selected_item_name();
            print!("  {}", label);
            println!("\n> {}", selected_item_name);
            Some(selected_index - 2)
        }
    }
}
