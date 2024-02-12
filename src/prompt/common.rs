use anyhow::Result;
use std::collections::HashMap;

use crate::{git, open_ai};

pub struct PromptData<'a> {
    pub previous_commit_messages: Vec<String>,
    pub diff: HashMap<String, git::FileDiff>,

    pub openai_client: &'a open_ai::OpenAIClient,
    pub model: String,
    pub refinement_prompt: Option<String>,
}

impl<'a> PromptData<'a> {
    pub fn from_repo(
        repo: &git2::Repository,
        openai_client: &'a open_ai::OpenAIClient,
        model: &str,
    ) -> Result<PromptData<'a>> {
        let previous_commit_messages = git::get_log_messages(repo, 10)?;
        let diff = git::get_staged_diff(repo)?;

        let result = PromptData {
            previous_commit_messages,
            diff,
            openai_client,
            model: String::from(model),
            refinement_prompt: None,
        };

        Ok(result)
    }

    pub fn update_refinement_prompt(&mut self, prompt: String) {
        self.refinement_prompt = Some(prompt);
    }
}

pub trait PromptGenerator {
    fn generate_results(&self, num_results: usize) -> Result<Vec<String>>;
}

pub fn model_propmpt_length(model: &str) -> usize {
    (match model {
        "gpt-3.5-turbo" => 4096,
        "gpt-3.5-turbo-0125" => 16385,
        "gpt-4" => 8192,
        _ => 4096,
    }) * 4 // 1 token is roughly 4 symbols accoridng to OpenAI docs
}
