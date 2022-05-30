use crate::editor::Editor;
use crate::error::Result;
use color_eyre::owo_colors::OwoColorize;
use std::fmt::Display;
use terminal_menu::{button, menu, mut_menu, run};
use tracing::warn;

pub struct InputReader<E: Editor> {
    editor: E,
}

impl<E: Editor> InputReader<E> {
    pub fn new(editor: E) -> Self {
        Self { editor }
    }

    pub fn request_user_input<M>(&mut self, request_msg: &M) -> Result<String>
    where
        M: Display + ?Sized,
    {
        println!("  {}", request_msg);
        self.editor.read_line()
    }

    pub fn ask_confirmation<M>(&mut self, confirmation_msg: &M) -> bool
    where
        M: Display + ?Sized,
    {
        loop {
            let result = self
                .request_user_input::<M>(confirmation_msg)
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
}

pub fn request_select_item_in_list<'a>(
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
