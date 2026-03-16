use anyhow::{Context, Result};
use async_trait::async_trait;
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use eventsource_stream::Eventsource;
use tokio::sync::mpsc;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Message {
    pub role: String,
    pub content: String,
}

#[async_trait]
pub trait ModelProvider: Send + Sync {
    async fn complete_stream(
        &self,
        messages: Vec<Message>,
    ) -> Result<mpsc::Receiver<Result<String>>>;
    
    async fn chat(
        &self,
        messages: Vec<Message>,
    ) -> Result<String>;
}

#[derive(Clone)]
pub struct GeminiProvider {
    api_key: String,
    model: String,
    use_bearer: bool,
    client: reqwest::Client,
}

impl GeminiProvider {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            model: "gemini-2.5-flash".into(),
            use_bearer: false,
            client: reqwest::Client::new(),
        }
    }

    pub fn with_model(mut self, model: &str) -> Self {
        self.model = model.into();
        self
    }

    /// Use OAuth Bearer token auth instead of ?key= URL parameter.
    pub fn with_bearer_auth(mut self) -> Self {
        self.use_bearer = true;
        self
    }

    fn build_url(&self, action: &str) -> String {
        if self.use_bearer {
            format!(
                "https://generativelanguage.googleapis.com/v1beta/models/{}:{}",
                self.model, action
            )
        } else {
            format!(
                "https://generativelanguage.googleapis.com/v1beta/models/{}:{}?key={}",
                self.model, action, self.api_key
            )
        }
    }

    fn apply_auth(&self, req: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        if self.use_bearer {
            req.header("Authorization", format!("Bearer {}", self.api_key))
        } else {
            req
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
    async fn complete_stream(
        &self,
        messages: Vec<Message>,
    ) -> Result<mpsc::Receiver<Result<String>>> {
        let (tx, rx) = mpsc::channel(100);
        let url = if self.use_bearer {
            format!(
                "https://generativelanguage.googleapis.com/v1beta/models/{}:streamGenerateContent?alt=sse",
                self.model
            )
        } else {
            format!(
                "https://generativelanguage.googleapis.com/v1beta/models/{}:streamGenerateContent?alt=sse&key={}",
                self.model, self.api_key
            )
        };

        let contents = messages.into_iter().map(|m| {
            GeminiContent {
                role: if m.role == "user" { "user".to_string() } else { "model".to_string() },
                parts: vec![GeminiPart { text: m.content }],
            }
        }).collect();

        let request = GeminiRequest { contents };
        let client = self.client.clone();
        let use_bearer = self.use_bearer;
        let api_key = self.api_key.clone();

        tokio::spawn(async move {
            let mut req = client.post(url).json(&request);
            if use_bearer {
                req = req.header("Authorization", format!("Bearer {}", api_key));
            }
            let res = req.send().await;

            match res {
                Ok(response) => {
                    let status = response.status();
                    if !status.is_success() {
                        // Read error body for diagnostics
                        let body = response.text().await.unwrap_or_default();
                        let _ = tx.send(Err(anyhow::anyhow!(
                            "Gemini API error ({}): {}", status, &body[..body.len().min(500)]
                        ))).await;
                        return;
                    }
                    let mut stream = response.bytes_stream().eventsource();
                    while let Some(event) = stream.next().await {
                        match event {
                            Ok(e) => {
                                if let Ok(resp) = serde_json::from_str::<GeminiResponse>(&e.data) {
                                    if let Some(candidate) = resp.candidates.get(0) {
                                        if let Some(part) = candidate.content.parts.get(0) {
                                            let _ = tx.send(Ok(part.text.clone())).await;
                                        }
                                    }
                                } else {
                                    // Try to surface Gemini error messages from SSE data
                                    let _ = tx.send(Err(anyhow::anyhow!(
                                        "Gemini stream parse error: {}", &e.data[..e.data.len().min(300)]
                                    ))).await;
                                    break;
                                }
                            }
                            Err(err) => {
                                let _ = tx.send(Err(anyhow::anyhow!("Stream error: {}", err))).await;
                                break;
                            }
                        }
                    }
                }
                Err(err) => {
                    let _ = tx.send(Err(anyhow::anyhow!("Request error: {}", err))).await;
                }
            }
        });

        Ok(rx)
    }

    async fn chat(&self, messages: Vec<Message>) -> Result<String> {
        let url = self.build_url("generateContent");

        let contents = messages.into_iter().map(|m| {
            GeminiContent {
                role: if m.role == "user" { "user".to_string() } else { "model".to_string() },
                parts: vec![GeminiPart { text: m.content }],
            }
        }).collect();

        let request = GeminiRequest { contents };

        let resp = self.apply_auth(self.client.post(&url))
            .json(&request)
            .send()
            .await?;

        let status = resp.status();
        let body = resp.text().await?;

        if !status.is_success() {
            anyhow::bail!("Gemini API error ({}): {}", status, body);
        }

        let parsed: GeminiResponse = serde_json::from_str(&body)
            .with_context(|| format!("Failed to parse Gemini response: {}", &body[..body.len().min(500)]))?;

        let text = parsed.candidates.get(0)
            .context("No candidates in response")?
            .content.parts.get(0)
            .context("No parts in candidate")?
            .text.clone();
            
        Ok(text)
    }
}

#[derive(Clone)]
pub struct AnthropicProvider {
    api_key: String,
    model: String,
    client: reqwest::Client,
}

impl AnthropicProvider {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            model: "claude-sonnet-4-20250514".into(),
            client: reqwest::Client::new(),
        }
    }

    pub fn with_model(mut self, model: &str) -> Self {
        self.model = model.into();
        self
    }
}

#[derive(Serialize)]
struct AnthropicRequest {
    model: String,
    messages: Vec<AnthropicMessage>,
    max_tokens: u32,
    stream: bool,
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

#[derive(Deserialize)]
#[serde(tag = "type")]
enum AnthropicStreamEvent {
    #[serde(rename = "content_block_delta")]
    Delta { delta: AnthropicDelta },
    #[serde(other)]
    Unknown,
}

#[derive(Deserialize)]
struct AnthropicDelta {
    text: String,
}

#[async_trait]
impl ModelProvider for AnthropicProvider {
    async fn complete_stream(
        &self,
        messages: Vec<Message>,
    ) -> Result<mpsc::Receiver<Result<String>>> {
        let (tx, rx) = mpsc::channel(100);
        let url = "https://api.anthropic.com/v1/messages";

        let anthropic_messages: Vec<AnthropicMessage> = messages.into_iter().map(|m| {
            AnthropicMessage {
                role: m.role,
                content: m.content,
            }
        }).collect();

        let request = AnthropicRequest {
            model: self.model.clone(),
            messages: anthropic_messages,
            max_tokens: 4096,
            stream: true,
        };

        let client = self.client.clone();
        let api_key = self.api_key.clone();

        tokio::spawn(async move {
            let res = client.post(url)
                .header("x-api-key", &api_key)
                .header("anthropic-version", "2023-06-01")
                .json(&request)
                .send()
                .await;

            match res {
                Ok(response) => {
                    let mut stream = response.bytes_stream().eventsource();
                    while let Some(event) = stream.next().await {
                        match event {
                            Ok(e) => {
                                if let Ok(AnthropicStreamEvent::Delta { delta }) = serde_json::from_str::<AnthropicStreamEvent>(&e.data) {
                                    let _ = tx.send(Ok(delta.text)).await;
                                }
                            }
                            Err(err) => {
                                let _ = tx.send(Err(anyhow::anyhow!("Stream error: {}", err))).await;
                                break;
                            }
                        }
                    }
                }
                Err(err) => {
                    let _ = tx.send(Err(anyhow::anyhow!("Request error: {}", err))).await;
                }
            }
        });

        Ok(rx)
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
            model: self.model.clone(),
            messages: anthropic_messages,
            max_tokens: 4096,
            stream: false,
        };

        let response = self.client.post(url)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .json(&request)
            .send()
            .await?
            .json::<AnthropicResponse>()
            .await?;

        let text = response.content.get(0)
            .context("No content in response")?
            .text.clone();
            
        Ok(text)
    }
}

// ── OpenRouter Provider (OpenAI-compatible API) ─────────────────────────────

#[derive(Clone)]
pub struct OpenRouterProvider {
    api_key: String,
    model: String,
    client: reqwest::Client,
}

impl OpenRouterProvider {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            model: "anthropic/claude-sonnet-4".into(),
            client: reqwest::Client::new(),
        }
    }

    pub fn with_model(mut self, model: &str) -> Self {
        self.model = model.into();
        self
    }
}

#[derive(Serialize)]
struct OpenRouterRequest {
    model: String,
    messages: Vec<OpenRouterMessage>,
    stream: bool,
}

#[derive(Serialize)]
struct OpenRouterMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct OpenRouterResponse {
    choices: Vec<OpenRouterChoice>,
}

#[derive(Deserialize)]
struct OpenRouterChoice {
    message: Option<OpenRouterMsg>,
    delta: Option<OpenRouterDelta>,
}

#[derive(Deserialize)]
struct OpenRouterMsg {
    content: String,
}

#[derive(Deserialize)]
struct OpenRouterDelta {
    content: Option<String>,
}

#[async_trait]
impl ModelProvider for OpenRouterProvider {
    async fn complete_stream(
        &self,
        messages: Vec<Message>,
    ) -> Result<mpsc::Receiver<Result<String>>> {
        let (tx, rx) = mpsc::channel(100);
        let url = "https://openrouter.ai/api/v1/chat/completions";

        let or_messages: Vec<OpenRouterMessage> = messages.into_iter().map(|m| {
            OpenRouterMessage {
                role: m.role,
                content: m.content,
            }
        }).collect();

        let request = OpenRouterRequest {
            model: self.model.clone(),
            messages: or_messages,
            stream: true,
        };

        let client = self.client.clone();
        let api_key = self.api_key.clone();

        tokio::spawn(async move {
            let res = client.post(url)
                .header("Authorization", format!("Bearer {}", api_key))
                .header("HTTP-Referer", "https://github.com/open-biz/guv-code")
                .header("X-Title", "GuvCode")
                .json(&request)
                .send()
                .await;

            match res {
                Ok(response) => {
                    let mut stream = response.bytes_stream().eventsource();
                    while let Some(event) = stream.next().await {
                        match event {
                            Ok(e) => {
                                if e.data == "[DONE]" {
                                    break;
                                }
                                if let Ok(resp) = serde_json::from_str::<OpenRouterResponse>(&e.data) {
                                    if let Some(choice) = resp.choices.get(0) {
                                        if let Some(delta) = &choice.delta {
                                            if let Some(content) = &delta.content {
                                                let _ = tx.send(Ok(content.clone())).await;
                                            }
                                        }
                                    }
                                }
                            }
                            Err(err) => {
                                let _ = tx.send(Err(anyhow::anyhow!("Stream error: {}", err))).await;
                                break;
                            }
                        }
                    }
                }
                Err(err) => {
                    let _ = tx.send(Err(anyhow::anyhow!("Request error: {}", err))).await;
                }
            }
        });

        Ok(rx)
    }

    async fn chat(&self, messages: Vec<Message>) -> Result<String> {
        let url = "https://openrouter.ai/api/v1/chat/completions";

        let or_messages = messages.into_iter().map(|m| {
            OpenRouterMessage {
                role: m.role,
                content: m.content,
            }
        }).collect();

        let request = OpenRouterRequest {
            model: self.model.clone(),
            messages: or_messages,
            stream: false,
        };

        let response: OpenRouterResponse = self.client.post(url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("HTTP-Referer", "https://github.com/open-biz/guv-code")
            .header("X-Title", "GuvCode")
            .json(&request)
            .send()
            .await?
            .json()
            .await?;

        let text = response.choices.get(0)
            .context("No choices in response")?
            .message.as_ref()
            .context("No message in choice")?
            .content.clone();

        Ok(text)
    }
}
