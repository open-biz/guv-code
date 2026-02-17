mod config;
mod index;
mod llm;
mod agent;
mod ast;

use clap::{Parser, Subcommand};
use anyhow::{Context, Result};
use config::Config;
use index::RepoIndex;
use agent::{ScoutAgent, CoderAgent};
use llm::{GeminiProvider, AnthropicProvider};
use std::env;
use std::fs;
use inquire::Text;
use indicatif::{ProgressBar, ProgressStyle};

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
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let mut config = Config::load()?;

    match cli.command {
        Commands::Auth { gemini, anthropic } => {
            if let Some(key) = gemini {
                config.keys.gemini = Some(key);
                println!("Gemini API key updated.");
            }
            if let Some(key) = anthropic {
                config.keys.anthropic = Some(key);
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
        Commands::Chat { message } => {
            let repo_path = env::current_dir()?;
            let gemini_key = config.keys.gemini.clone().context("Gemini API key not set. Run `guv auth --gemini <key>`")?;
            let anthropic_key = config.keys.anthropic.clone().context("Anthropic API key not set. Run `guv auth --anthropic <key>`")?;

            let scout_provider = GeminiProvider::new(gemini_key);
            let coder_provider = AnthropicProvider::new(anthropic_key);

            let scout = ScoutAgent::new(&scout_provider);
            let coder = CoderAgent::new(&coder_provider);

            let mut current_message = message;

            loop {
                let query = match current_message.take() {
                    Some(m) => m,
                    None => {
                        match Text::new("Guv'nor?").prompt() {
                            Ok(m) if !m.trim().is_empty() => m,
                            _ => break,
                        }
                    }
                };

                if query == "exit" || query == "quit" {
                    break;
                }

                let pb = ProgressBar::new_spinner();
                pb.set_style(ProgressStyle::default_spinner()
                    .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"])
                    .template("{spinner:.blue} {msg}")?);
                
                pb.set_message("Updating index...");
                let mut index = RepoIndex::load_or_create(&repo_path)?;
                let _changed = index.update(&repo_path)?;
                index.save(&repo_path)?;

                pb.set_message("Scouting for relevant files...");
                let relevant_files = scout.find_files(&index, &query).await?;
                
                pb.set_message("Generating edits...");
                let mut file_contents = Vec::new();
                for path_str in relevant_files {
                    let path = repo_path.join(&path_str);
                    if path.exists() {
                        let content = fs::read_to_string(path)?;
                        file_contents.push((path_str, content));
                    }
                }

                let edits = coder.generate_edits(&query, file_contents).await?;
                pb.finish_and_clear();

                println!("\n--- PROPOSED EDITS ---\n{}\n---------------------\n", edits);
            }
        }
    }

    Ok(())
}
