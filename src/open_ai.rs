use core::fmt;

use serde::Deserialize;

use crate::ARGS;

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

#[derive(Debug)]
pub struct JsonParseErrorData {
    pub serde_error: serde_json::Error,
    pub raw_json: String,
}

#[derive(Debug)]
pub enum OpenAIError {
    JsonParseError(JsonParseErrorData),
    ReqwestError(reqwest::Error),
}

impl fmt::Display for OpenAIError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            OpenAIError::JsonParseError(data) => write!(
                f,
                "Failed to parse JSON: {}\nOriginal JSON: {}",
                data.serde_error, data.raw_json
            ),
            OpenAIError::ReqwestError(e) => write!(f, "Reqwest error: {}", e),
        }
    }
}

impl std::error::Error for OpenAIError {}

pub fn get_completion(body: serde_json::Value) -> Result<ChatCompletionResponse, OpenAIError> {
    let api_key = &ARGS.openai_api_key;

    let client = reqwest::blocking::Client::new();
    let response_body = client
        .post(CHAT_COMPLETION_URL)
        .json(&body)
        .header("Authorization", format!("Bearer {}", api_key))
        .send()
        .map_err(OpenAIError::ReqwestError)?
        .text()
        .map_err(OpenAIError::ReqwestError)?;

    serde_json::from_str(&response_body).map_err(|e| {
        OpenAIError::JsonParseError(JsonParseErrorData {
            serde_error: e,
            raw_json: response_body,
        })
    })
}
