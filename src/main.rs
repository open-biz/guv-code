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
            println!("Updating index...");
            let mut index = RepoIndex::load_or_create(&repo_path)?;
            let _changed = index.update(&repo_path)?;
            index.save(&repo_path)?;
            
            let query = message.context("Interactive mode not yet implemented. Please provide a message.")?;
            
            let gemini_key = config.keys.gemini.clone().context("Gemini API key not set. Run `guv auth --gemini <key>`")?;
            let anthropic_key = config.keys.anthropic.clone().context("Anthropic API key not set. Run `guv auth --anthropic <key>`")?;

            let scout_provider = GeminiProvider::new(gemini_key);
            let coder_provider = AnthropicProvider::new(anthropic_key);

            let scout = ScoutAgent::new(&scout_provider);
            let coder = CoderAgent::new(&coder_provider);

            println!("Scouting for relevant files...");
            let relevant_files = scout.find_files(&index, &query).await?;
            println!("Relevant files: {:?}", relevant_files);

            let mut file_contents = Vec::new();
            for path_str in relevant_files {
                let path = repo_path.join(&path_str);
                if path.exists() {
                    let content = fs::read_to_string(path)?;
                    file_contents.push((path_str, content));
                }
            }

            println!("Generating edits...");
            let edits = coder.generate_edits(&query, file_contents).await?;
            println!("--- PROPOSED EDITS ---\n{}\n---------------------", edits);
        }
    }

    Ok(())
}
