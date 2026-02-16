use anyhow::{Context, Result};
use async_trait::async_trait;
use futures::{Stream, StreamExt};
use std::pin::Pin;
use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub struct Message {
    pub role: String,
    pub content: String,
}

#[async_trait]
pub trait ModelProvider: Send + Sync {
    async fn chat_stream(
        &self,
        messages: Vec<Message>,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<String>> + Send>>>;
    
    async fn chat(
        &self,
        messages: Vec<Message>,
    ) -> Result<String>;
}

pub struct GeminiProvider {
    api_key: String,
    client: reqwest::Client,
}

impl GeminiProvider {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            client: reqwest::Client::new(),
        }
    }
}

#[derive(Serialize)]
struct GeminiRequest {
    contents: Vec<GeminiContent>,
}

#[derive(Serialize)]
struct GeminiContent {
    role: String,
    parts: Vec<GeminiPart>,
}

#[derive(Serialize)]
struct GeminiPart {
    text: String,
}

#[derive(Deserialize)]
struct GeminiResponse {
    candidates: Vec<GeminiCandidate>,
}

#[derive(Deserialize)]
struct GeminiCandidate {
    content: GeminiContentResponse,
}

#[derive(Deserialize)]
struct GeminiContentResponse {
    parts: Vec<GeminiPartResponse>,
}

#[derive(Deserialize)]
struct GeminiPartResponse {
    text: String,
}

#[async_trait]
impl ModelProvider for GeminiProvider {
    async fn chat_stream(
        &self,
        _messages: Vec<Message>,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<String>> + Send>>> {
        // Basic implementation for now, streaming for Gemini is a bit complex
        // let's start with non-streaming or simple mock for now if it's too much
        anyhow::bail!("Streaming not yet implemented for Gemini")
    }

    async fn chat(&self, messages: Vec<Message>) -> Result<String> {
        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/gemini-1.5-pro:generateContent?key={}",
            self.api_key
        );

        let contents = messages.into_iter().map(|m| {
            GeminiContent {
                role: if m.role == "user" { "user".to_string() } else { "model".to_string() },
                parts: vec![GeminiPart { text: m.content }],
            }
        }).collect();

        let request = GeminiRequest { contents };

        let response = self.client.post(url)
            .json(&request)
            .send()
            .await?
            .json::<GeminiResponse>()
            .await?;

        response.candidates.get(0)
            .context("No candidates in response")?
            .content.parts.get(0)
            .context("No parts in candidate")?
            .text.clone().into()
            .pipe(Ok)
    }
}

pub struct AnthropicProvider {
    api_key: String,
    client: reqwest::Client,
}

impl AnthropicProvider {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            client: reqwest::Client::new(),
        }
    }
}

#[derive(Serialize)]
struct AnthropicRequest {
    model: String,
    messages: Vec<AnthropicMessage>,
    max_tokens: u32,
}

#[derive(Serialize)]
struct AnthropicMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct AnthropicResponse {
    content: Vec<AnthropicContent>,
}

#[derive(Deserialize)]
struct AnthropicContent {
    text: String,
}

#[async_trait]
impl ModelProvider for AnthropicProvider {
    async fn chat_stream(
        &self,
        _messages: Vec<Message>,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<String>> + Send>>> {
        anyhow::bail!("Streaming not yet implemented for Anthropic")
    }

    async fn chat(&self, messages: Vec<Message>) -> Result<String> {
        let url = "https://api.anthropic.com/v1/messages";

        let anthropic_messages = messages.into_iter().map(|m| {
            AnthropicMessage {
                role: m.role,
                content: m.content,
            }
        }).collect();

        let request = AnthropicRequest {
            model: "claude-3-7-sonnet-latest".to_string(),
            messages: anthropic_messages,
            max_tokens: 4096,
        };

        let response = self.client.post(url)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .json(&request)
            .send()
            .await?
            .json::<AnthropicResponse>()
            .await?;

        response.content.get(0)
            .context("No content in response")?
            .text.clone().into()
            .pipe(Ok)
    }
}

// Simple pipe helper because I used it above
trait Pipe {
    fn pipe<F, R>(self, f: F) -> R where F: FnOnce(Self) -> R, Self: Sized;
}
impl<T> Pipe for T {
    fn pipe<F, R>(self, f: F) -> R where F: FnOnce(Self) -> R, Self: Sized { f(self) }
}
