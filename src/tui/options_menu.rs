use std::io::{self};

use console::{style, Key, Term};

use super::common::{cycle_decrement, cycle_increment, ReadAction};

pub struct OptionsMenu {
    message: String,
    options: Vec<String>,
}

enum OptionsMenuAction {
    PrevItem,
    NextItem,
    Confirm,
}

impl ReadAction<OptionsMenuAction> for OptionsMenu {
    fn match_action(&self, key: Key) -> Option<OptionsMenuAction> {
        match key {
            // Char code is for CTRL-P
            Key::ArrowUp | Key::Char('\u{10}') => Some(OptionsMenuAction::PrevItem),
            // Char code is for CTRL-N
            Key::ArrowDown | Key::Tab | Key::Char('\u{E}') => Some(OptionsMenuAction::NextItem),
            Key::Enter => Some(OptionsMenuAction::Confirm),
            _ => None,
        }
    }
}

impl OptionsMenu {
    pub fn new(message: String, options: Vec<String>) -> OptionsMenu {
        OptionsMenu { message, options }
    }

    fn format_message(&self) -> String {
        format!(
            "{} {} {}",
            style("?").cyan(),
            style(&self.message).bold(),
            style("[use arrows to move, enter to select]").white()
        )
    }

    fn format_selected_option(&self, option: &String) -> String {
        style(format!("> {}", option)).cyan().to_string()
    }

    fn format_unselected_option(&self, option: &String) -> String {
        format!("  {}", option)
    }

    pub fn read_answer(&self) -> io::Result<usize> {
        let term = Term::stderr();

        let mut selected: usize = 0;
        let options_count = self.options.len();

        loop {
            term.write_line(&self.format_message())?;

            for (i, option) in self.options.iter().enumerate() {
                let option = match i {
                    x if x == selected => self.format_selected_option(option),
                    _ => self.format_unselected_option(option),
                };

                term.write_line(&option)?;
            }

            match self.read_action(&term)? {
                OptionsMenuAction::PrevItem => cycle_decrement(&mut selected, &options_count),
                OptionsMenuAction::NextItem => cycle_increment(&mut selected, &options_count),
                OptionsMenuAction::Confirm => {
                    return Ok(selected);
                }
            }

            term.clear_last_lines(options_count + 1)?;
        }
    }
}
