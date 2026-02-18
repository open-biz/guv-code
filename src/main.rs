mod config;
mod index;
mod llm;
mod agent;
mod ast;
mod diff;
mod git;
mod tui;

use clap::{Parser, Subcommand};
use anyhow::{Context, Result};
use config::Config;
use git::GitManager;
use std::env;

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
    let mut config = Config::load()?;

    match cli.command {
        Commands::Auth { gemini, anthropic } => {
            if let Some(ref key) = gemini {
                config.keys.gemini = Some(key.clone());
                println!("Gemini API key updated.");
            }
            if let Some(ref key) = anthropic {
                config.keys.anthropic = Some(key.clone());
                println!("Anthropic API key updated.");
            }
            config.save()?;
            if gemini.is_none() && anthropic.is_none() {
                println!("Current keys: Gemini: {:?}, Anthropic: {:?}", 
                    config.keys.gemini.as_ref().map(|_| "****"),
                    config.keys.anthropic.as_ref().map(|_| "****")
                );
            }
        }
        Commands::Budget { limit, status } => {
            if let Some(l) = limit {
                config.budget.limit = l;
                println!("Budget limit updated to ${:.2}", l);
                config.save()?;
            }
            if status || limit.is_none() {
                println!("Budget Status:");
                println!("  Limit:     ${:.2}", config.budget.limit);
                println!("  Consumed:  ${:.2}", config.budget.consumed);
                println!("  Remaining: ${:.2}", config.budget.limit - config.budget.consumed);
            }
        }
        Commands::Chat { message: _ } => {
            tui::run_tui().context("Failed to run TUI")?;
        }
        Commands::Undo => {
            let repo_path = env::current_dir()?;
            if GitManager::is_repo(&repo_path) {
                GitManager::undo(&repo_path)?;
                println!("Undone last edit.");
            } else {
                println!("Not a git repository.");
            }
        }
    }

    Ok(())
}
