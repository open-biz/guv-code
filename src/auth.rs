use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use std::fs;
use std::path::PathBuf;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::net::TcpListener;

// ── Auth Types (modeled after gemini-cli's AuthType enum) ───────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AuthType {
    /// Google OAuth 2.0 with PKCE (browser-based "Sign in with Google")
    GoogleOAuth,
    /// Gemini API key (GEMINI_API_KEY env var)
    GeminiApiKey,
    /// Anthropic API key (ANTHROPIC_API_KEY env var)
    AnthropicApiKey,
    /// OpenRouter API key (via OAuth or OPENROUTER_API_KEY env var)
    OpenRouter,
    /// Vertex AI (GOOGLE_CLOUD_PROJECT + GOOGLE_CLOUD_LOCATION)
    VertexAi,
}

// ── OAuth Token ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthToken {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub token_type: String,
    pub expires_at: Option<u64>, // Unix timestamp in seconds
    pub scope: Option<String>,
}

impl OAuthToken {
    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            // 5 minute buffer for clock skew
            now + 300 >= expires_at
        } else {
            false
        }
    }
}

// ── PKCE Parameters ─────────────────────────────────────────────────────────

pub struct PKCEParams {
    pub code_verifier: String,
    pub code_challenge: String,
    pub state: String,
}

impl PKCEParams {
    pub fn generate() -> Self {
        use rand::Rng;
        let mut rng = rand::thread_rng();

        // Generate code verifier (43-128 chars, base64url-safe)
        let verifier_bytes: Vec<u8> = (0..64).map(|_| rng.r#gen::<u8>()).collect();
        let code_verifier = base64_url_encode(&verifier_bytes);

        // Generate code challenge using SHA256
        let mut hasher = Sha256::new();
        hasher.update(code_verifier.as_bytes());
        let challenge_bytes = hasher.finalize();
        let code_challenge = base64_url_encode(&challenge_bytes);

        // Generate state for CSRF protection
        let state_bytes: Vec<u8> = (0..16).map(|_| rng.r#gen::<u8>()).collect();
        let state = base64_url_encode(&state_bytes);

        Self {
            code_verifier,
            code_challenge,
            state,
        }
    }
}

fn base64_url_encode(bytes: &[u8]) -> String {
    use base64::Engine;
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(bytes)
}

// ── OAuth Flow Config ───────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct OAuthFlowConfig {
    pub client_id: String,
    pub client_secret: Option<String>,
    pub auth_url: String,
    pub token_url: String,
    pub scopes: Vec<String>,
    pub redirect_uri: Option<String>,
}

// Google OAuth credentials from gemini-cli
// (installed application — client secret is not treated as secret per Google docs)
const GOOGLE_OAUTH_CLIENT_ID: &str = "YOUR_CLIENT_ID";
const GOOGLE_OAUTH_CLIENT_SECRET: &str = "YOUR_CLIENT_SECRET";

impl OAuthFlowConfig {
    /// Google OAuth config matching gemini-cli's credentials and scopes.
    pub fn google() -> Self {
        Self {
            client_id: GOOGLE_OAUTH_CLIENT_ID.into(),
            client_secret: Some(GOOGLE_OAUTH_CLIENT_SECRET.into()),
            auth_url: "https://accounts.google.com/o/oauth2/v2/auth".into(),
            token_url: "https://oauth2.googleapis.com/token".into(),
            scopes: vec![
                "https://www.googleapis.com/auth/cloud-platform".into(),
                "https://www.googleapis.com/auth/userinfo.email".into(),
                "https://www.googleapis.com/auth/userinfo.profile".into(),
            ],
            redirect_uri: None,
        }
    }
}

impl Default for OAuthFlowConfig {
    fn default() -> Self {
        Self::google()
    }
}

// ── Credential Storage ──────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredCredentials {
    pub auth_type: AuthType,
    pub token: Option<OAuthToken>,
    pub api_key: Option<String>,
    pub updated_at: u64,
}

fn credentials_path() -> Result<PathBuf> {
    let home = directories::UserDirs::new()
        .context("Could not find home directory")?
        .home_dir()
        .to_path_buf();
    Ok(home.join(".guv").join("credentials.json"))
}

pub fn load_credentials() -> Result<Option<StoredCredentials>> {
    let path = credentials_path()?;
    if !path.exists() {
        return Ok(None);
    }
    let content = fs::read_to_string(&path).context("Failed to read credentials")?;
    let creds: StoredCredentials =
        serde_json::from_str(&content).context("Failed to parse credentials")?;
    Ok(Some(creds))
}

pub fn save_credentials(creds: &StoredCredentials) -> Result<()> {
    let path = credentials_path()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).context("Failed to create credentials directory")?;
    }
    let content = serde_json::to_string_pretty(creds).context("Failed to serialize credentials")?;

    // Write with restricted permissions (0o600)
    fs::write(&path, &content).context("Failed to write credentials")?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = fs::Permissions::from_mode(0o600);
        fs::set_permissions(&path, perms).ok();
    }

    Ok(())
}

pub fn clear_credentials() -> Result<()> {
    let path = credentials_path()?;
    if path.exists() {
        fs::remove_file(&path).context("Failed to remove credentials")?;
    }
    Ok(())
}

// ── Auth Detection (modeled after gemini-cli's getAuthTypeFromEnv) ──────────

pub fn detect_auth_from_env() -> Option<AuthType> {
    if std::env::var("GOOGLE_GENAI_USE_GCA").ok().as_deref() == Some("true") {
        return Some(AuthType::GoogleOAuth);
    }
    if std::env::var("GOOGLE_GENAI_USE_VERTEXAI").ok().as_deref() == Some("true") {
        return Some(AuthType::VertexAi);
    }
    if std::env::var("OPENROUTER_API_KEY").is_ok() {
        return Some(AuthType::OpenRouter);
    }
    if std::env::var("GEMINI_API_KEY").is_ok() {
        return Some(AuthType::GeminiApiKey);
    }
    if std::env::var("ANTHROPIC_API_KEY").is_ok() {
        return Some(AuthType::AnthropicApiKey);
    }
    None
}

// ── OAuth 2.0 + PKCE Flow ───────────────────────────────────────────────────

/// Start a local HTTP callback server for OAuth redirect.
/// Returns (port, receiver for authorization code).
pub async fn start_callback_server(
    expected_state: &str,
) -> Result<(u16, tokio::sync::oneshot::Receiver<String>)> {
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let port = listener.local_addr()?.port();

    let (tx, rx) = tokio::sync::oneshot::channel();
    let state = expected_state.to_string();

    tokio::spawn(async move {
        if let Ok((stream, _)) = listener.accept().await {
            let (reader, writer) = stream.into_split();
            let mut buf_reader = BufReader::new(reader);
            let mut request_line = String::new();
            if buf_reader.read_line(&mut request_line).await.is_ok() {
                // Parse: GET /oauth/callback?code=XXX&state=YYY HTTP/1.1
                if let Some(path) = request_line.split_whitespace().nth(1) {
                    if let Ok(url) = url::Url::parse(&format!("http://localhost{}", path)) {
                        let code = url
                            .query_pairs()
                            .find(|(k, _)| k == "code")
                            .map(|(_, v)| v.to_string());
                        let recv_state = url
                            .query_pairs()
                            .find(|(k, _)| k == "state")
                            .map(|(_, v)| v.to_string());

                        // Drain remaining headers before writing response
                        loop {
                            let mut line = String::new();
                            match buf_reader.read_line(&mut line).await {
                                Ok(0) => break,
                                Ok(_) if line.trim().is_empty() => break,
                                Err(_) => break,
                                _ => {}
                            }
                        }

                        use tokio::io::AsyncWriteExt;
                        let mut w = writer;

                        if let (Some(code), Some(s)) = (code, recv_state) {
                            if s == state {
                                let html = "<html><body style=\"font-family:system-ui;text-align:center;padding:60px\">\
                                    <h1>Signed in!</h1>\
                                    <p>GuvCode received your credentials. You can close this tab.</p>\
                                    </body></html>";
                                let resp = format!(
                                    "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                                    html.len(), html
                                );
                                let _ = w.write_all(resp.as_bytes()).await;
                                let _ = w.shutdown().await;
                                let _ = tx.send(code);
                            } else {
                                let body = "State mismatch — possible CSRF. Please try again.";
                                let resp = format!(
                                    "HTTP/1.1 400 Bad Request\r\nContent-Type: text/plain\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                                    body.len(), body
                                );
                                let _ = w.write_all(resp.as_bytes()).await;
                                let _ = w.shutdown().await;
                            }
                        } else {
                            let body = "Missing code or state parameter.";
                            let resp = format!(
                                "HTTP/1.1 400 Bad Request\r\nContent-Type: text/plain\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                                body.len(), body
                            );
                            let _ = w.write_all(resp.as_bytes()).await;
                            let _ = w.shutdown().await;
                        }
                        return;
                    }
                }
            }
            // Fallback: couldn't parse, close cleanly
            drop(writer);
        }
    });

    Ok((port, rx))
}

/// Build the Google OAuth authorization URL.
pub fn build_auth_url(config: &OAuthFlowConfig, pkce: &PKCEParams, port: u16) -> String {
    let redirect_uri = config
        .redirect_uri
        .clone()
        .unwrap_or_else(|| format!("http://localhost:{}/oauth/callback", port));

    let scopes = config.scopes.join(" ");

    format!(
        "{}?client_id={}&response_type=code&redirect_uri={}&state={}&code_challenge={}&code_challenge_method=S256&scope={}&access_type=offline&prompt=consent",
        config.auth_url,
        urlencoding::encode(&config.client_id),
        urlencoding::encode(&redirect_uri),
        urlencoding::encode(&pkce.state),
        urlencoding::encode(&pkce.code_challenge),
        urlencoding::encode(&scopes),
    )
}

/// Exchange authorization code for tokens.
pub async fn exchange_code_for_token(
    config: &OAuthFlowConfig,
    code: &str,
    code_verifier: &str,
    port: u16,
) -> Result<OAuthToken> {
    let redirect_uri = config
        .redirect_uri
        .clone()
        .unwrap_or_else(|| format!("http://localhost:{}/oauth/callback", port));

    let mut params = vec![
        ("grant_type", "authorization_code".to_string()),
        ("code", code.to_string()),
        ("redirect_uri", redirect_uri),
        ("code_verifier", code_verifier.to_string()),
        ("client_id", config.client_id.clone()),
    ];

    if let Some(secret) = &config.client_secret {
        params.push(("client_secret", secret.clone()));
    }

    let client = reqwest::Client::new();
    let resp = client
        .post(&config.token_url)
        .form(&params)
        .header("Accept", "application/json")
        .send()
        .await
        .context("Token exchange request failed")?;

    if !resp.status().is_success() {
        let body = resp.text().await.unwrap_or_default();
        anyhow::bail!("Token exchange failed: {}", body);
    }

    let body: serde_json::Value = resp.json().await.context("Failed to parse token response")?;

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let expires_in = body["expires_in"].as_u64().unwrap_or(3600);

    Ok(OAuthToken {
        access_token: body["access_token"]
            .as_str()
            .context("Missing access_token")?
            .to_string(),
        refresh_token: body["refresh_token"].as_str().map(|s| s.to_string()),
        token_type: body["token_type"]
            .as_str()
            .unwrap_or("Bearer")
            .to_string(),
        expires_at: Some(now + expires_in),
        scope: body["scope"].as_str().map(|s| s.to_string()),
    })
}

/// Refresh an access token using a refresh token.
pub async fn refresh_access_token(
    config: &OAuthFlowConfig,
    refresh_token: &str,
) -> Result<OAuthToken> {
    let mut params = vec![
        ("grant_type", "refresh_token".to_string()),
        ("refresh_token", refresh_token.to_string()),
        ("client_id", config.client_id.clone()),
    ];

    if let Some(secret) = &config.client_secret {
        params.push(("client_secret", secret.clone()));
    }

    let client = reqwest::Client::new();
    let resp = client
        .post(&config.token_url)
        .form(&params)
        .header("Accept", "application/json")
        .send()
        .await
        .context("Token refresh request failed")?;

    if !resp.status().is_success() {
        let body = resp.text().await.unwrap_or_default();
        anyhow::bail!("Token refresh failed: {}", body);
    }

    let body: serde_json::Value = resp.json().await.context("Failed to parse refresh response")?;

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let expires_in = body["expires_in"].as_u64().unwrap_or(3600);

    Ok(OAuthToken {
        access_token: body["access_token"]
            .as_str()
            .context("Missing access_token")?
            .to_string(),
        refresh_token: body["refresh_token"]
            .as_str()
            .map(|s| s.to_string())
            .or_else(|| Some(refresh_token.to_string())),
        token_type: body["token_type"]
            .as_str()
            .unwrap_or("Bearer")
            .to_string(),
        expires_at: Some(now + expires_in),
        scope: body["scope"].as_str().map(|s| s.to_string()),
    })
}

// ── OpenRouter OAuth (PKCE flow modeled after aider's onboarding.py) ────────

/// Start OpenRouter OAuth PKCE flow.
/// Returns the API key on success.
pub async fn run_openrouter_oauth() -> Result<String> {
    let pkce = PKCEParams::generate();

    // Start local callback server
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let port = listener.local_addr()?.port();
    let callback_url = format!("http://localhost:{}/callback/guv", port);

    let (tx, rx) = tokio::sync::oneshot::channel();
    let expected_path = "/callback/guv";

    tokio::spawn(async move {
        if let Ok((stream, _)) = listener.accept().await {
            let (reader, writer) = stream.into_split();
            let mut buf_reader = BufReader::new(reader);
            let mut request_line = String::new();
            if buf_reader.read_line(&mut request_line).await.is_ok() {
                if let Some(path) = request_line.split_whitespace().nth(1) {
                    if path.starts_with(expected_path) {
                        if let Ok(url) = url::Url::parse(&format!("http://localhost{}", path)) {
                            let code = url
                                .query_pairs()
                                .find(|(k, _)| k == "code")
                                .map(|(_, v)| v.to_string());
                            if let Some(code) = code {
                                // Send success HTML response
                                let response = "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\n\r\n\
                                    <html><body><h1>Success!</h1>\
                                    <p>GuvCode has received the authentication code. \
                                    You can close this browser tab.</p></body></html>";
                                use tokio::io::AsyncWriteExt;
                                let mut w = writer;
                                let _ = w.write_all(response.as_bytes()).await;
                                let _ = tx.send(code);
                            }
                        }
                    }
                }
            }
        }
    });

    // Build OpenRouter auth URL
    let auth_url = format!(
        "https://openrouter.ai/auth?callback_url={}&code_challenge={}&code_challenge_method=S256",
        urlencoding::encode(&callback_url),
        urlencoding::encode(&pkce.code_challenge),
    );

    open_browser(&auth_url)?;

    // Wait for callback (5 minute timeout)
    let code = tokio::time::timeout(
        std::time::Duration::from_secs(300),
        rx,
    )
    .await
    .map_err(|_| anyhow::anyhow!("OpenRouter OAuth timed out (5 minutes)"))?
    .map_err(|_| anyhow::anyhow!("OpenRouter OAuth callback cancelled"))?;

    // Exchange code for API key
    let api_key = exchange_openrouter_code(&code, &pkce.code_verifier).await?;

    // Save to credentials
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let creds = StoredCredentials {
        auth_type: AuthType::OpenRouter,
        token: None,
        api_key: Some(api_key.clone()),
        updated_at: now,
    };
    save_credentials(&creds)?;

    // Also save to ~/.guv/openrouter-key.env for auto-loading
    let key_path = credentials_path()?
        .parent()
        .map(|p| p.join("openrouter-key.env"))
        .context("Could not determine key file path")?;
    fs::write(&key_path, format!("OPENROUTER_API_KEY=\"{}\"\n", api_key))?;

    // Set for current session
    // SAFETY: We are single-threaded at this point in the OAuth flow
    unsafe { std::env::set_var("OPENROUTER_API_KEY", &api_key); }

    Ok(api_key)
}

/// Exchange OpenRouter authorization code for an API key.
async fn exchange_openrouter_code(code: &str, code_verifier: &str) -> Result<String> {
    let client = reqwest::Client::new();
    let resp = client
        .post("https://openrouter.ai/api/v1/auth/keys")
        .header("Content-Type", "application/json")
        .json(&serde_json::json!({
            "code": code,
            "code_verifier": code_verifier,
            "code_challenge_method": "S256",
        }))
        .send()
        .await
        .context("OpenRouter code exchange request failed")?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        anyhow::bail!("OpenRouter code exchange failed ({}): {}", status, body);
    }

    let body: serde_json::Value = resp.json().await.context("Failed to parse OpenRouter response")?;
    let key = body["key"]
        .as_str()
        .context("'key' not found in OpenRouter response")?
        .to_string();

    Ok(key)
}

/// Open a URL in the user's default browser.
pub fn open_browser(url: &str) -> Result<()> {
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(url)
            .spawn()
            .context("Failed to open browser with xdg-open")?;
    }
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(url)
            .spawn()
            .context("Failed to open browser")?;
    }
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("cmd")
            .args(&["/c", "start", url])
            .spawn()
            .context("Failed to open browser")?;
    }
    Ok(())
}
