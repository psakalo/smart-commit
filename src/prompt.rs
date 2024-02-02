use std::{error, fmt};

use serde::Deserialize;

use crate::{git, open_ai};

#[derive(Debug, Deserialize)]
struct CommitMessagesResponse {
    messages: Vec<String>,
}

pub struct CommitCompletionData {
    // Previous commit message to use as context
    pub previous_messages: Vec<String>,

    pub diff: Vec<String>,

    model: String,
    results_count: usize,
    prompt: Option<String>,
    messages: Vec<(String, String)>,
}

#[derive(Debug)]
pub enum PromptError {
    NoStagedFiles,
    OpenAiNoChoices,
    OpenAiNoContent,
    /// OpenAI returned a response, but it was not in the expected format, for example unexpected number
    /// of lines
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

impl error::Error for PromptError {}

impl CommitCompletionData {
    fn get_length_for_model(model: &String) -> usize {
        let length: usize = match model.as_str() {
            "gpt-3.5-turbo" => 4096,
            "gpt-3.5-turbo-0125" => 16385,
            "gpt-4" => 8192,
            _ => 4096,
        } * 4; // 1 token is roughly 4 symbols accoridng to OpenAI docs

        // Leave some space for the prompt
        length - 4096
    }

    pub fn from_path(
        dir: &String,
        model: &String,
        results_count: usize,
    ) -> Result<CommitCompletionData, PromptError> {
        let previous_messages =
            git::get_log_messages(dir, 10).map_err(|_| PromptError::GitError)?;
        let diff = git::get_staged_diff(dir, Self::get_length_for_model(model))
            .map_err(|_| PromptError::NoStagedFiles)?;

        let mut result = CommitCompletionData {
            previous_messages,
            diff,
            model: model.clone(),
            prompt: None,
            results_count,
            messages: vec![],
        };

        result.prompt = Some(result.build_prompt(results_count)?);

        Ok(result)
    }

    fn build_prompt(&self, num: usize) -> Result<String, PromptError> {
        let mut prompt = String::new();

        prompt.push_str(&format!("You are a software developer working on a project, you have just made changes to the project. 
Produce JSON with field \"messages\" containing an array of {num} different commit messages which best describe provided changes.
If previous commit message examples are provided, use them as a style guide.
Use semantic commit message convention only if you can detect usage from provided previous commits.
Ant other user message should be considered as refinement of the prompt and response should be JSON."));

        // Add previous commit messages
        if !self.previous_messages.is_empty() {
            prompt.push_str("\n\n# Previous commit messages:\n");

            self.previous_messages.iter().for_each(|message| {
                prompt.push_str(&format!("  - {}\n", message));
            });
        }

        // Add diff
        if self.diff.is_empty() {
            return Err(PromptError::NoStagedFiles);
        }

        prompt.push_str("\n\n# Changes in git diff format:\n");

        self.diff.iter().for_each(|file_diff| {
            prompt.push_str(&format!("\n```diff\n{}\n```\n", file_diff));
        });

        Ok(prompt)
    }

    fn build_request_body(
        &self,
        model: &str,
        messages: Vec<(String, String)>,
    ) -> serde_json::Value {
        serde_json::json!({
            "model": model,
            "messages": messages.iter().map(|(role, content)| {
                serde_json::json!({
                    "role": role,
                    "content": content
                })
            }).collect::<Vec<_>>()
        })
    }

    fn extract_results(
        &self,
        completion: open_ai::ChatCompletionResponse,
    ) -> Result<((String, String), Vec<String>), PromptError> {
        let choice = completion
            .choices
            .first()
            .ok_or(PromptError::OpenAiNoChoices)?;

        let (role, content) = (
            choice.message.role.clone(),
            choice
                .message
                .content
                .clone()
                .ok_or(PromptError::OpenAiNoContent)?,
        );

        let parsed_response: CommitMessagesResponse =
            serde_json::from_str(&content).map_err(|e| {
                PromptError::OpenAiError(open_ai::OpenAIError::JsonParseError(
                    open_ai::JsonParseErrorData {
                        serde_error: e,
                        raw_json: content.clone(),
                    },
                ))
            })?;

        match parsed_response.messages.len() {
            x if x == self.results_count => Ok(((role, content), parsed_response.messages)),
            _ => Err(PromptError::OpenAiWrongContent(format!(
                "Expected {} lines, got {}",
                self.results_count, content
            ))),
        }
    }

    pub fn complete_commit_messages(&mut self) -> Result<Vec<String>, PromptError> {
        let body = self.build_request_body(
            &self.model,
            vec![(String::from("user"), self.prompt.clone().unwrap())],
        );

        let completion = open_ai::get_completion(body).map_err(PromptError::OpenAiError)?;

        let (message, results) = self.extract_results(completion)?;
        self.messages.push(message);

        Ok(results)
    }

    pub fn refine_commit_messages(&mut self, prompt: &str) -> Result<Vec<String>, PromptError> {
        let mut messages = vec![(String::from("user"), self.prompt.clone().unwrap())];
        messages.append(self.messages.clone().as_mut());
        messages.push((String::from("user"), prompt.to_string()));

        let body = self.build_request_body(&self.model, messages);
        let completion = open_ai::get_completion(body).map_err(PromptError::OpenAiError)?;

        let (message, results) = self.extract_results(completion)?;
        self.messages
            .push((String::from("user"), prompt.to_string()));
        self.messages.push(message);

        Ok(results)
    }
}
