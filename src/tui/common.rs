use std::io::{self};

use console::{Key, Term};

/// Generic trait that implements read_action method that accepts map of possible keys and enum with
/// corresponding actions
pub trait ReadAction<T> {
    /// Matches key with action
    fn match_action(&self, key: Key) -> Option<T>;

    /// Read user input until one of action is not matched, and then return matched action. Blocks
    fn read_action(&self, term: &Term) -> io::Result<T> {
        loop {
            let key = term.read_key()?;

            if let Some(action) = self.match_action(key) {
                return Ok(action);
            }
        }
    }
}

pub fn cycle_increment(num: &mut usize, max: &usize) {
    *num = (*num + 1) % max;
}

pub fn cycle_decrement(num: &mut usize, max: &usize) {
    *num = if *num == 0 { max - 1 } else { *num - 1 };
}
