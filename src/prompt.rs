use std::{error, fmt};

use crate::{git, open_ai};

pub struct CommitCompletionData {
    // Previous commit message to use as context
    pub previous_messages: Vec<String>,
    pub diff: Vec<String>,
    pub model: String,

    refinement_prompt: Option<String>,
}

#[derive(Debug)]
pub enum PromptError {
    NoStagedFiles,
    OpenAiNoChoices,
    OpenAiNoContent,
    /// OpenAI returned a response, but it was not in the expected format,
    /// for example unexpected number of lines
    OpenAiWrongContent(String),
    OpenAiError(open_ai::OpenAIError),
    GitError,
}

impl fmt::Display for PromptError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            PromptError::NoStagedFiles => write!(f, "No staged files"),
            PromptError::OpenAiNoChoices => write!(f, "OpenAI returned no choices"),
            PromptError::OpenAiNoContent => write!(f, "OpenAI returned no content"),
            PromptError::OpenAiWrongContent(m) => write!(f, "OpenAI returned wrong content: {}", m),
            PromptError::OpenAiError(e) => write!(f, "OpenAI error: {}", e),
            PromptError::GitError => write!(f, "Git error"),
        }
    }
}

const PROMPT: &str = include_str!("prompt.txt");

impl error::Error for PromptError {}

impl CommitCompletionData {
    fn get_length_for_model(model: &str) -> usize {
        let length: usize = match model {
            "gpt-3.5-turbo" => 4096,
            "gpt-3.5-turbo-0125" => 16385,
            "gpt-4" => 8192,
            _ => 4096,
        } * 4; // 1 token is roughly 4 symbols accoridng to OpenAI docs

        // Leave some space for the prompt
        length - 2048
    }

    pub fn from_path(dir: &str, model: &str) -> Result<CommitCompletionData, PromptError> {
        let previous_messages =
            git::get_log_messages(dir, 10).map_err(|_| PromptError::GitError)?;
        let diff = git::get_staged_diff(dir, Self::get_length_for_model(model))
            .map_err(|_| PromptError::NoStagedFiles)?;

        let result = CommitCompletionData {
            previous_messages,
            diff,
            model: model.to_string(),
            refinement_prompt: None,
        };

        Ok(result)
    }

    fn build_prompt(&self) -> Result<String, PromptError> {
        if self.diff.is_empty() {
            return Err(PromptError::NoStagedFiles);
        }

        let prompt = PROMPT.to_string();

        let prompt = prompt.replace(
            "{{ previous_messages_block }}",
            if self.previous_messages.is_empty() {
                "".to_string()
            } else {
                let mut block = String::from("# Previous commit messages:\n");

                self.previous_messages.iter().for_each(|message| {
                    block.push_str(&format!("  - {}\n", message));
                });

                block
            }
            .as_str(),
        );

        let prompt = prompt.replace(
            "{{ diff }}",
            {
                let mut diffs = String::new();

                self.diff.iter().for_each(|file_diff| {
                    diffs.push_str(&format!("\n```diff\n{}\n```\n", file_diff));
                });

                diffs
            }
            .as_str(),
        );

        Ok(prompt)
    }

    fn build_request_body(
        &self,
        model: &str,
        num_results: usize,
    ) -> Result<serde_json::Value, PromptError> {
        let content = self.build_prompt()?;

        let mut body = serde_json::json!({
            "model": model,
            "n": num_results,
            "temperature": 1,
            "messages": [
                {
                    "role": "user",
                    "content": content
                },
            ]
        });

        if let Some(refinement_prompt) = &self.refinement_prompt {
            let messages = body.as_object_mut().unwrap()["messages"]
                .as_array_mut()
                .unwrap();
            messages.push(serde_json::json!({
                "role": "user",
                "content": refinement_prompt
            }));
        }

        Ok(body)
    }

    fn extract_results(
        &self,
        completion: open_ai::ChatCompletionResponse,
    ) -> Result<Vec<String>, PromptError> {
        completion
            .choices
            .into_iter()
            .map(|choice| -> Result<String, PromptError> {
                let text = choice.message.content.ok_or(PromptError::OpenAiNoContent)?;

                // Return first line only
                Ok(text
                    .lines()
                    .next()
                    .ok_or(PromptError::OpenAiWrongContent(text.clone().to_string()))?
                    .to_string())
            })
            .collect::<Result<Vec<String>, PromptError>>()
    }

    pub fn complete_commit_messages(
        &mut self,
        num_results: usize,
        refinement_prompt: Option<String>,
    ) -> Result<Vec<String>, PromptError> {
        self.refinement_prompt = refinement_prompt;

        let body = self.build_request_body(&self.model, num_results)?;
        let completion = open_ai::get_completion(body).map_err(PromptError::OpenAiError)?;
        let results = self.extract_results(completion)?;
        Ok(results)
    }
}
