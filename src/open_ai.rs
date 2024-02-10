use anyhow::anyhow;
use anyhow::Result;

use inquire::Text;
use serde::Deserialize;

use crate::args::ARGS;

const CHAT_COMPLETION_URL: &str = "https://api.openai.com/v1/chat/completions";

#[derive(Deserialize)]
pub struct Function {
    pub name: String,
    pub arguments: String,
}

#[derive(Deserialize)]
pub struct ToolCall {
    pub id: String,
    pub r#type: String,
    pub function: Function,
}

#[derive(Deserialize)]
pub struct Message {
    pub content: Option<String>,
    pub tool_calls: Option<Vec<ToolCall>>,
    pub role: String,
}

#[derive(Deserialize)]
pub struct Usage {
    pub completion_tokens: u32,
    pub prompt_tokens: u32,
    pub total_tokens: u32,
}

#[derive(Deserialize)]
pub struct Choice {
    pub finish_reason: Option<String>,
    pub index: u32,
    pub message: Message,
}

#[derive(Deserialize)]
pub struct ChatCompletionResponse {
    pub id: String,
    pub choices: Vec<Choice>,
    pub created: u32,
    pub model: String,
    pub system_fingerprint: Option<String>,
    pub object: String,
    pub usage: Option<Usage>,
}

pub struct OpenAIKey {
    key: String,
}

impl OpenAIKey {
    /// Create a new OpenAIKey instance, if key does not exist, will ask user for input
    pub fn new_ensure() -> Result<OpenAIKey> {
        // If key is provided as command line argument, always prioritize it
        if let (false, Some(key)) = (ARGS.force_key_prompt, &ARGS.openai_api_key) {
            return Ok(OpenAIKey {
                key: key.to_string(),
            });
        }

        // Try to get key from keyring
        let keyring_entry = keyring::Entry::new("smart-commit", "openai_api_key")?;
        if let (false, Ok(key)) = (ARGS.force_key_prompt, keyring_entry.get_password()) {
            return Ok(OpenAIKey { key });
        }

        // Read key from user and upate keyring
        let key = Text::new("OpenAI API key:").prompt()?;
        keyring_entry.set_password(&key)?;

        Ok(OpenAIKey { key })
    }

    pub fn get_key(&self) -> &str {
        &self.key
    }
}

pub struct OpenAIClient {
    api_key: OpenAIKey,
}

impl OpenAIClient {
    pub fn new(api_key: OpenAIKey) -> OpenAIClient {
        OpenAIClient { api_key }
    }

    pub fn get_completion(
        &self,
        body: serde_json::Value, // More convenient serde_json::Value type
    ) -> Result<ChatCompletionResponse> {
        // Return anyhow::Result

        let client = reqwest::blocking::Client::new();
        let response_body = client
            .post(CHAT_COMPLETION_URL)
            .json(&body)
            .header(
                "Authorization",
                format!("Bearer {}", self.api_key.get_key()),
            )
            .send()
            .and_then(|res| res.text()) // Use and_then for chaining
            .map_err(|err| anyhow!("Reqwest Error: {}", err))?; // Convert to anyhow::Error

        serde_json::from_str(&response_body)
            .map_err(|e| anyhow!("JSON Parse Error: {}\nRaw JSON: {}", e, response_body))
        // More informative error message
    }
}
