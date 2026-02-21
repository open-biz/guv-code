mod config;
mod index;
mod llm;
mod agent;
mod ast;
mod diff;
mod git;
mod tui;

use clap::{Parser, Subcommand};
use config::Config;
use git::GitManager;
use std::env;
use owo_colors::OwoColorize;
use miette::{Result, miette};

#[derive(Parser)]
#[command(name = "guv")]
#[command(about = "GUV-Code: Right away, Guv'nor.", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
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
    },
    /// Manage and view token budget
    Budget {
        /// Set a new budget limit in USD
        #[arg(short, long)]
        limit: Option<f64>,
        /// View current consumption and limit
        #[arg(short, long)]
        status: bool,
    },
    /// Start an AI-powered chat session
    Chat {
        /// The message to send
        message: Option<String>,
    },
    /// Undo the last AI edit
    Undo,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let mut config = Config::load().map_err(|e| miette!("{}", e))?;

    match cli.command {
        Commands::Auth { gemini, anthropic } => {
            if let Some(ref key) = gemini {
                config.keys.gemini = Some(key.clone());
                println!("{} Gemini API key updated.", "✅".green());
            }
            if let Some(ref key) = anthropic {
                config.keys.anthropic = Some(key.clone());
                println!("{} Anthropic API key updated.", "✅".green());
            }
            config.save().map_err(|e| miette!("{}", e))?;
            if gemini.is_none() && anthropic.is_none() {
                println!("{}:", "Current keys".bold().blue());
                println!("  Gemini:    {}", config.keys.gemini.as_ref().map(|_| "****").unwrap_or("Not set").yellow());
                println!("  Anthropic: {}", config.keys.anthropic.as_ref().map(|_| "****").unwrap_or("Not set").yellow());
            }
        }
        Commands::Budget { limit, status } => {
            if let Some(l) = limit {
                config.budget.limit = l;
                println!("{} Budget limit updated to ${:.2}", "✅".green(), l);
                config.save().map_err(|e| miette!("{}", e))?;
            }
            if status || limit.is_none() {
                println!("{}:", "Budget Status".bold().blue());
                println!("  Limit:     ${:.2}", config.budget.limit);
                println!("  Consumed:  ${:.2}", config.budget.consumed);
                println!("  Remaining: ${:.2}", (config.budget.limit - config.budget.consumed).green());
            }
        }
        Commands::Chat { message: _ } => {
            println!("🎩 {}", "GUV-Code Initialized. Right away, Guv'nor.".bold().cyan());
            tui::run_tui().await.map_err(|e| miette!("{}", e))?;
        }
        Commands::Undo => {
            let repo_path = env::current_dir().map_err(|e| miette!("{}", e))?;
            if GitManager::is_repo(&repo_path) {
                GitManager::undo(&repo_path).map_err(|e| miette!("{}", e))?;
                println!("{} Undone last edit.", "✅".green());
            } else {
                println!("{} Not a git repository.", "❌".red());
            }
        }
    }

    Ok(())
}
