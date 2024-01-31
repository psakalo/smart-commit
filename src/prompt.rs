use std::error::Error;

use crate::git;

pub struct CommitCompletionData {
    // Previous commit message to use as context
    pub previous_messages: Vec<String>,

    pub diff: Vec<String>,
}

impl CommitCompletionData {
    pub fn from_path(dir: &String) -> Result<CommitCompletionData, Box<dyn Error>> {
        let previous_messages = git::get_log_messages(dir, 10)?;
        let diff = git::get_staged_diff(dir)?;

        Ok(CommitCompletionData {
            previous_messages,
            diff,
        })
    }

    pub fn build_prompt(&self) -> Result<String, Box<dyn Error>> {
        let mut prompt = String::new();

        prompt.push_str("You are a software developer working on a project. You have just made changes to the project. 
Write a commit message for the commit. If previous commit examples are supplied, use them as style guide.
Use semantic commit messages only if you can detect usage from previous commits. Commit message should be one line exactly.");

        // Add previous commit messages
        if self.previous_messages.len() > 0 {
            prompt.push_str("\n\n# Previous commit messages:\n");

            self.previous_messages.iter().for_each(|message| {
                prompt.push_str(&format!("  - {}\n", message));
            });
        }

        // Add diff
        if self.diff.len() == 0 {
            Err("No diff found")?;
        }

        prompt.push_str("\n\n# Changes:\n");

        self.diff.iter().for_each(|file_diff| {
            prompt.push_str(&format!("\n```diff\n{}\n```\n", file_diff));
        });

        Ok(prompt)
    }
}
