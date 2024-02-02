use std::io::{self};

use console::{style, Key, Term};

use super::common::ReadAction;

pub struct Input {
    message: String,
}

enum InputAction {
    Confirm,
    Char(char),
}

impl ReadAction<InputAction> for Input {
    fn match_action(&self, key: Key) -> Option<InputAction> {
        match key {
            Key::Enter => Some(InputAction::Confirm),
            Key::Char(char) => Some(InputAction::Char(char)),
            Key::Backspace => Some(InputAction::Char('\x08')),
            Key::Del => Some(InputAction::Char('\x7F')),
            _ => None,
        }
    }
}

impl Input {
    pub fn new(message: String) -> Input {
        Input { message }
    }

    fn format_message(&self) -> String {
        format!("{} {}: ", style("?").cyan(), style(&self.message).bold(),)
    }

    pub fn read_answer(&self) -> io::Result<String> {
        let term = Term::stderr();

        term.write_str(&self.format_message())?;

        let mut result = String::new();

        loop {
            match self.read_action(&term)? {
                InputAction::Confirm => {
                    term.write_line("")?;
                    return Ok(result);
                }
                InputAction::Char(c) => {
                    result.push(c);
                    term.write_str(&c.to_string())?;
                }
            }
        }
    }
}
