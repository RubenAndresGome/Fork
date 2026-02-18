use serde::{Deserialize, Serialize};
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};
use serde_json::json;

#[derive(Serialize, Deserialize)]
struct ChatCompletionResponse {
    choices: Vec<Choice>,
    usage: Usage,
}

#[derive(Serialize, Deserialize)]
struct Choice {
    message: Message,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Message {
    pub role: String,
    pub content: String,
}

#[derive(Serialize, Deserialize)]
struct Usage {
    total_tokens: u32,
}

pub async fn send_chat_completion(
    api_key: &str,
    messages: Vec<Message>,
) -> Result<(String, f64), String> {
    let client = reqwest::Client::new();
    
    let body = json!({
        "model": "gpt-3.5-turbo",
        "messages": messages
    });

    let res = client.post("https://api.openai.com/v1/chat/completions")
        .header(CONTENT_TYPE, "application/json")
        .header(AUTHORIZATION, format!("Bearer {}", api_key))
        .json(&body)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !res.status().is_success() {
        return Err(format!("API Error: {}", res.status()));
    }

    let response: ChatCompletionResponse = res.json().await.map_err(|e| e.to_string())?;
    
    let content = response.choices.first()
        .map(|c| c.message.content.clone())
        .ok_or("No content in response")?;

    // Estimación costo GPT-3.5 Turbo: $0.0005 / 1K input, $0.0015 / 1K output.
    // Simplificación: $0.0015 / 1K total tokens medio
    let cost = (response.usage.total_tokens as f64 / 1000.0) * 0.0015;

    Ok((content, cost))
}
