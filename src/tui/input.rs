use std::io::{self};

use console::{style, Key, Term};

use super::common::ReadAction;

pub struct Input {
    message: String,
}

enum InputAction {
    Confirm,
    Char(char),
    OtherKey(Key),
}

impl ReadAction<InputAction> for Input {
    fn match_action(&self, key: Key) -> Option<InputAction> {
        match key {
            Key::Enter => Some(InputAction::Confirm),
            Key::Char(char) => Some(InputAction::Char(char)),
            key => Some(InputAction::OtherKey(key)),
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

    pub fn read_answer(&self, prefill: &Option<&str>) -> io::Result<String> {
        let term = Term::stderr();

        term.write_str(&self.format_message())?;

        let mut result = String::from(prefill.unwrap_or(""));

        if let Some(prefill) = prefill {
            term.write_str(prefill)?;
        }

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
                InputAction::OtherKey(Key::Backspace) => {
                    if result.pop().is_some() {
                        term.clear_chars(1)?;
                    }
                }
                _ => {}
            }
        }
    }
}
