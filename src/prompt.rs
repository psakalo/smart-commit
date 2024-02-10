use anyhow::Result;
use std::{collections::HashMap, error, fmt};

use crate::{args::ARGS, git, open_ai};

pub struct CommitCompletionData {
    // Previous commit messages to use as a context
    pub previous_messages: Vec<String>,
    pub diff: HashMap<String, git::FileDiff>,
    pub model: String,

    pub refinement_prompt: Option<String>,
}

#[derive(Debug)]
pub enum PromptError {
    NoStagedFiles,
    OpenAiNoChoices,
    OpenAiNoContent,
    /// OpenAI returned a response, but it was not in the expected format,
    /// for example unexpected number of lines
    OpenAiWrongContent(String),
    GitError,
}

impl fmt::Display for PromptError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            PromptError::NoStagedFiles => write!(f, "No staged files"),
            PromptError::OpenAiNoChoices => write!(f, "OpenAI returned no choices"),
            PromptError::OpenAiNoContent => write!(f, "OpenAI returned no content"),
            PromptError::OpenAiWrongContent(m) => write!(f, "OpenAI returned wrong content: {}", m),
            PromptError::GitError => write!(f, "Git error"),
        }
    }
}

static SYSTEM_PROMPT: &str = "You are a CLI program designed to generate clear, concise, and informative git commit messages for users based on provided diffs. 
Your response should be a short one-line string, that concisely summarizes the changes made.
When crafting commit messages, consider the context of the change, its purpose, and its impact on the project.
Focus on action verbs, clear descriptions, and the specific area of the project affected by the changes.";

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
        let diff = git::get_staged_diff(dir).map_err(|_| PromptError::NoStagedFiles)?;

        let result = CommitCompletionData {
            previous_messages,
            diff,
            model: model.to_string(),
            refinement_prompt: None,
        };

        Ok(result)
    }

    fn build_request_body(
        &self,
        model: &str,
        num_results: usize,
    ) -> Result<serde_json::Value, PromptError> {
        if self.diff.is_empty() {
            return Err(PromptError::NoStagedFiles);
        }

        let mut system_prompt = String::from(SYSTEM_PROMPT);

        if !self.previous_messages.is_empty() {
            system_prompt.push_str("\nUse the following previous commits as a style guide for consistency and clarity:\n");
            self.previous_messages.iter().for_each(|message| {
                system_prompt.push_str(&format!("  - {}\n", message));
            });
            system_prompt.push('\n');
        }

        if let Some(refinement_prompt) = &self.refinement_prompt {
            system_prompt.push_str("Additionally, consider following requirements:\n");
            system_prompt.push_str(refinement_prompt);
        }

        let mut user_prompt = String::new();
        user_prompt.push_str("# Git diff:\n");
        self.diff.iter().for_each(|(_path, file_diff)| {
            user_prompt.push_str(&format!("```diff\n{}\n```\n", file_diff.formatted_diff));
        });

        let body = serde_json::json!({
            "model": model,
            "n": num_results,
            "stop": "\n",
            "messages": [
                {
                    "role": "system",
                    "content": system_prompt
                },
                {
                    "role": "user",
                    "content": user_prompt
                },
            ]
        });

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
                    .trim()
                    .to_string())
            })
            .collect::<Result<Vec<String>, PromptError>>()
    }

    pub fn complete_commit_messages(
        &mut self,
        openai_client: &open_ai::OpenAIClient,
        num_results: usize,
        refinement_prompt: Option<String>,
    ) -> Result<Vec<String>> {
        self.refinement_prompt = refinement_prompt;

        let body = self.build_request_body(&self.model, num_results)?;
        if ARGS.debug {
            eprintln!("{}", console::style("\n[DEBUG] System prompt:").blue());
            eprintln!(
                "{}\n",
                body["messages"][0]["content"]
                    .to_string()
                    .replace("\\n", "\n")
            );
            eprintln!("{}", console::style("\n[DEBUG] User prompt:").blue());
            eprintln!(
                "{}\n",
                body["messages"][1]["content"]
                    .to_string()
                    .replace("\\n", "\n")
            );
        }

        let completion = openai_client.get_completion(body)?;
        let results = self.extract_results(completion)?;
        Ok(results)
    }
}
