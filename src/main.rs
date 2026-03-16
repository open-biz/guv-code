mod auth;
mod clipboard;
mod config;
mod index;
mod llm;
mod git;
mod terminal;
mod agent_logic;
mod orchestrator;
mod ui;

use clap::{Parser, Subcommand};
use config::Config;
use git::GitManager;
use std::env;
use owo_colors::OwoColorize;
use miette::{Result, miette};
use auth::AuthType;
use ui::app::TuiOptions;

#[derive(Parser)]
#[command(name = "guv")]
#[command(about = "GuvCode: Right away, Guv'nor.", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Initial prompt to send to the agent on startup
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    prompt: Vec<String>,

    /// Start in YOLO mode (auto-approve everything)
    #[arg(short = 'y', long)]
    yolo: bool,

    /// Start in plan-only mode
    #[arg(long)]
    plan: bool,

    /// Show tools pane on startup
    #[arg(long)]
    tools: bool,

    /// Override model (e.g. "gemini-2.5-pro", "claude-sonnet")
    #[arg(short = 'm', long)]
    model: Option<String>,
}

#[derive(Subcommand)]
enum Commands {
    /// Configure API keys and authentication
    Auth {
        /// Set Gemini API key
        #[arg(short, long)]
        gemini: Option<String>,
        /// Set Anthropic API key
        #[arg(short, long)]
        anthropic: Option<String>,
        /// Save to local .guvcode instead of global config
        #[arg(short, long)]
        local: bool,
        /// Sign in with Google OAuth (browser-based)
        #[arg(long)]
        login: bool,
        /// Clear stored credentials and log out
        #[arg(long)]
        logout: bool,
    },
    /// Manage and view token budget
    Budget {
        /// Set a new budget limit in USD
        #[arg(short, long)]
        limit: Option<f64>,
        /// View current consumption and limit
        #[arg(short, long)]
        status: bool,
        /// Save to local .guvcode instead of global config
        #[arg(short, long)]
        local: bool,
    },
    /// Undo the last AI edit
    Undo,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let mut config = Config::load().map_err(|e| miette!("{}", e))?;

    match cli.command {
        Some(Commands::Auth { gemini, anthropic, local, login, logout }) => {
            if logout {
                auth::clear_credentials().map_err(|e| miette!("{}", e))?;
                println!("{} Logged out. Credentials cleared.", "✅".green());
                return Ok(());
            }

            if login {
                println!("{}  Signing in with Google...", "🔐".bold());
                let oauth_config = auth::OAuthFlowConfig::default();
                let pkce = auth::PKCEParams::generate();
                let (port, code_rx) = auth::start_callback_server(&pkce.state)
                    .await
                    .map_err(|e| miette!("{}", e))?;
                let auth_url = auth::build_auth_url(&oauth_config, &pkce, port);

                println!("\nOpen this URL in your browser to authenticate:");
                println!("  {}", auth_url.bold().cyan());
                println!("\nWaiting for callback...");

                auth::open_browser(&auth_url).ok();

                let code = code_rx.await.map_err(|_| miette!("OAuth callback timed out or was cancelled"))?;

                println!("  {} Received authorization code", "✓".green());

                let token = auth::exchange_code_for_token(&oauth_config, &code, &pkce.code_verifier, port)
                    .await
                    .map_err(|e| miette!("{}", e))?;

                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();

                let creds = auth::StoredCredentials {
                    auth_type: AuthType::GoogleOAuth,
                    token: Some(token),
                    api_key: None,
                    updated_at: now,
                };
                auth::save_credentials(&creds).map_err(|e| miette!("{}", e))?;
                println!("{} Signed in with Google OAuth.", "✅".green());
                return Ok(());
            }

            if let Some(ref key) = gemini {
                config.keys.gemini = Some(key.clone());
                println!("{} Gemini API key updated.", "✅".green());
            }
            if let Some(ref key) = anthropic {
                config.keys.anthropic = Some(key.clone());
                println!("{} Anthropic API key updated.", "✅".green());
            }
            
            if local {
                config.save_local().map_err(|e| miette!("{}", e))?;
            } else {
                config.save_global().map_err(|e| miette!("{}", e))?;
            }

            if gemini.is_none() && anthropic.is_none() {
                // Show current auth status
                let env_auth = auth::detect_auth_from_env();
                let stored = auth::load_credentials().ok().flatten();

                println!("{}:", "Authentication Status".bold().blue());
                if let Some(ref creds) = stored {
                    println!("  Method:    {}", format!("{:?}", creds.auth_type).cyan());
                    let has_token = creds.token.as_ref().map(|t| !t.is_expired()).unwrap_or(false);
                    if has_token {
                        println!("  Token:     {}", "Valid ✓".green());
                    } else {
                        println!("  Token:     {}", "Expired or missing".yellow());
                    }
                } else if let Some(ref auth_type) = env_auth {
                    println!("  Method:    {} (from env)", format!("{:?}", auth_type).cyan());
                } else {
                    println!("  Method:    {}", "Not configured".yellow());
                }
                println!();
                println!("{}:", "API Keys".bold().blue());
                println!("  Gemini:    {}", config.keys.gemini.as_ref().map(|_| "****").unwrap_or("Not set").yellow());
                println!("  Anthropic: {}", config.keys.anthropic.as_ref().map(|_| "****").unwrap_or("Not set").yellow());
                println!();
                println!("{}:", "Options".bold().blue());
                println!("  guv auth --login     Sign in with Google OAuth");
                println!("  guv auth --logout    Clear stored credentials");
                println!("  guv auth -g <KEY>    Set Gemini API key");
                println!("  guv auth -a <KEY>    Set Anthropic API key");
            }
        }
        Some(Commands::Budget { limit, status, local }) => {
            if let Some(l) = limit {
                config.budget.limit = l;
                println!("{} Budget limit updated to ${:.2}", "✅".green(), l);
                if local {
                    config.save_local().map_err(|e| miette!("{}", e))?;
                } else {
                    config.save_global().map_err(|e| miette!("{}", e))?;
                }
            }
            if status || limit.is_none() {
                println!("{}:", "Budget Status".bold().blue());
                println!("  Limit:     ${:.2}", config.budget.limit);
                println!("  Consumed:  ${:.2}", config.budget.consumed);
                println!("  Remaining: ${:.2}", (config.budget.limit - config.budget.consumed).green());
            }
        }
        Some(Commands::Undo) => {
            let repo_path = env::current_dir().map_err(|e| miette!("{}", e))?;
            if GitManager::is_repo(&repo_path) {
                GitManager::auto_stage_all(&repo_path).map_err(|e| miette!("{}", e))?;
                GitManager::undo(&repo_path).map_err(|e| miette!("{}", e))?;
                println!("{} Undone last edit.", "✅".green());
            } else {
                println!("{} Not a git repository.", "❌".red());
            }
        }
        None => {
            // Build TUI options from CLI flags (gemini-cli style)
            let initial_prompt = if cli.prompt.is_empty() {
                None
            } else {
                Some(cli.prompt.join(" "))
            };
            let opts = TuiOptions {
                prompt: initial_prompt,
                yolo: cli.yolo,
                plan: cli.plan,
                model: cli.model,
                show_tools: cli.tools,
            };
            ui::app::start_tui(config, opts).await.map_err(|e| miette!("{}", e))?;
        }
    }

    Ok(())
}
