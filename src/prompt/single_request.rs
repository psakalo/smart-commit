use anyhow::{anyhow, bail, Result};

use crate::{args::ARGS, open_ai};

use super::common::{PromptData, PromptGenerator};

static SYSTEM_PROMPT: &str = "You are a CLI program designed to generate clear, concise, and informative git commit messages for users based on provided diffs. 
Your response should be a short one-line string, that concisely summarizes the changes made.
When crafting commit messages, consider the context of the change, its purpose, and its impact on the project.
Focus on action verbs, clear descriptions, and the specific area of the project affected by the changes.";

pub struct SingleRequestPrompt<'a> {
    data: PromptData<'a>,
}

impl<'a> SingleRequestPrompt<'a> {
    pub fn from(data: PromptData<'a>) -> SingleRequestPrompt<'a> {
        SingleRequestPrompt { data }
    }

    fn build_request_body(&self, model: &str, num_results: usize) -> Result<serde_json::Value> {
        if self.data.diff.is_empty() {
            bail!("No staged changes found");
        }

        let mut system_prompt = String::from(SYSTEM_PROMPT);

        if !self.data.previous_commit_messages.is_empty() {
            system_prompt.push_str("\nUse the following previous commits as a style guide for consistency and clarity:\n");
            self.data
                .previous_commit_messages
                .iter()
                .for_each(|message| {
                    system_prompt.push_str(&format!("  - {}\n", message));
                });
            system_prompt.push('\n');
        }

        if let Some(refinement_prompt) = &self.data.refinement_prompt {
            system_prompt.push_str("Additionally, consider following requirements:\n");
            system_prompt.push_str(refinement_prompt);
        }

        let mut user_prompt = String::new();
        user_prompt.push_str("# Git diff:\n");
        self.data.diff.iter().for_each(|(_path, file_diff)| {
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

    fn extract_results(&self, completion: open_ai::ChatCompletionResponse) -> Result<Vec<String>> {
        completion
            .choices
            .into_iter()
            .map(|choice| -> Result<String> {
                let text = choice
                    .message
                    .content
                    .ok_or(anyhow!("No content in message"))?;

                // Return first line only
                Ok(text
                    .lines()
                    .next()
                    .ok_or(anyhow!(text.clone().to_string()))?
                    .trim()
                    .to_string())
            })
            .collect::<Result<Vec<String>>>()
    }

    pub fn get_refinement_prompt(&self) -> Option<String> {
        self.data.refinement_prompt.clone()
    }

    pub fn update_refinement_prompt(&mut self, prompt: String) {
        self.data.update_refinement_prompt(prompt);
    }
}

impl<'a> PromptGenerator for SingleRequestPrompt<'a> {
    fn generate_results(&self, num_results: usize) -> Result<Vec<String>> {
        let body = self.build_request_body(&self.data.model, num_results)?;
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

        let completion = self.data.openai_client.get_completion(body)?;
        let results = self.extract_results(completion)?;
        Ok(results)
    }
}
