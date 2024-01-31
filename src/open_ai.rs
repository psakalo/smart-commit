use serde::Deserialize;

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
    pub finish_reason: String,
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
    pub usage: Usage,
}

pub fn get_completion(model: &str, prompt: &str) -> Result<Choice, Box<dyn std::error::Error>> {
    // TODO: figure out if caching makes sense here
    let api_key = std::env::var("OPENAI_API_KEY").or(Err("Failed to get OPENAI_API_KEY"))?;

    let body = serde_json::json!({
        "model": model,
        "messages": [
            {
                "role": "user",
                "content": prompt,
            }
        ],
    });

    let client = reqwest::blocking::Client::new();
    let response_body = client
        .post(CHAT_COMPLETION_URL)
        .json(&body)
        .header("Authorization", format!("Bearer {}", api_key))
        .send()?
        .text()?;
    let mut completion: ChatCompletionResponse = match serde_json::from_str(&response_body) {
        Ok(json) => json,
        Err(e) => Err(format!(
            "Failed to parse response: {}\nOriginal response: {}",
            e, response_body
        ))?,
    };

    match completion.choices.pop() {
        Some(choice) => Ok(choice),
        None => Err("Failed to get completion")?,
    }
}
